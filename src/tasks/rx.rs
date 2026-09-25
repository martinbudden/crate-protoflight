use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::WaitResult,
    watch::{Receiver, Sender, Watch},
};

#[cfg(feature = "autopilot")]
use radio_controllers::RcMode;
use radio_controllers::{Radio, Rates, RatesConfig, RcModes, RxConfig, RxRadio};
use static_cell::StaticCell;

use crate::{
    boards::{RadioUartRx, RadioUartTx},
    config::{
        ConfigItem, ConfigPublisher, ConfigSubscriber, FastConfigPublisher, config_publisher, config_subscriber,
        fast_config_publisher,
    },
    flight::{RcAdjustments, RxMessage},
};

static RX_CTX: StaticCell<RxContext> = StaticCell::new();

// Note, we use a `Watch` rather than a `Signal` since the receiver (`gyro_pid` task) uses `try_changed` to see if the value has changed.
const RX_WATCH_COUNT: usize = 3;
static RX_WATCH: Watch<CriticalSectionRawMutex, RxMessage, RX_WATCH_COUNT> = Watch::new();

type RxMessageSender = Sender<'static, CriticalSectionRawMutex, RxMessage, RX_WATCH_COUNT>;
fn rx_message_sender() -> RxMessageSender {
    RX_WATCH.sender()
}

pub type RxMessageReceiver = Receiver<'static, CriticalSectionRawMutex, RxMessage, RX_WATCH_COUNT>;

#[allow(clippy::expect_used)]
pub fn rx_message_receiver() -> RxMessageReceiver {
    RX_WATCH.receiver().expect("rx_receiver failed")
}

#[cfg(feature = "autopilot")]
use super::autopilot::{AutopilotReceiver, autopilot_receiver};

/// Context for the receiver task.
pub struct RxContext {
    pub radio: Radio,
    #[allow(unused)]
    pub uart_rx: RadioUartRx,
    #[allow(unused)]
    pub uart_tx: RadioUartTx,
    pub rx_message_sender: RxMessageSender,
    pub config_subscriber: ConfigSubscriber,
    /// To publish in-flight adjustments.
    pub config_publisher: ConfigPublisher,
    /// To publish in-flight adjustments.
    pub fast_config_publisher: FastConfigPublisher,
    pub rc_modes: RcModes,
    pub rates: Rates,
    pub rc_adjustments: RcAdjustments,
    pub buf: [u8; Self::BUF_SIZE],

    #[cfg(feature = "autopilot")]
    pub autopilot_receiver: AutopilotReceiver,
}

impl RxContext {
    const BUF_SIZE: usize = 128;

    pub fn new(uart_rx: RadioUartRx, uart_tx: RadioUartTx, rx_config: RxConfig, rates_config: RatesConfig) -> Self {
        let radio = Radio::new(rx_config.serial_rx_provider);
        Self {
            radio,
            uart_rx,
            uart_tx,
            rx_message_sender: rx_message_sender(),
            config_subscriber: config_subscriber(),
            config_publisher: config_publisher(),
            fast_config_publisher: fast_config_publisher(),
            rates: Rates::new(rates_config),
            rc_modes: RcModes::with_mac_arm(),
            rc_adjustments: RcAdjustments::new(),
            buf: [0u8; Self::BUF_SIZE],

            #[cfg(feature = "autopilot")]
            autopilot_receiver: autopilot_receiver(),
        }
    }
}

pub fn init(
    uart_rx: RadioUartRx,
    uart_tx: RadioUartTx,
    rx_config: RxConfig,
    rates: RatesConfig,
) -> &'static mut RxContext {
    RX_CTX.init(RxContext::new(uart_rx, uart_tx, rx_config, rates))
}

/// The rx task waits (with a timeout) for a packet from the radio and when one arrives it:
/// 1. Checks for any in-flight adjustments of rates.
/// 2. Updates the control modes using the AUX channel values.
/// 3. Creates a `FlightControl` message from the values in the radio packet.
/// 4. Checks if a `FlightControl` message has arrived from the Autopilot, and processes it.
/// 5. Sends the `FlightControl` message to the `gyro_pid` task.
/// If the timeout expires, then failsafe handling is invoked.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut RxContext) {
    let mut loop_count: u32 = 0;

    log::info!("          RX: task started");

    loop {
        // Fetch data from UART
        if let Ok(n) = ctx.read_packet().await {
            // Process the buffer byte-by-byte
            for &byte in &ctx.buf[..n] {
                // If a frame completes, process it immediately inside the stream
                if ctx.radio.on_byte_received(byte) {
                    let rx_frame = ctx.radio.rx_frame();

                    // TODO: we need to do some failsafe checking here.
                    let failsafe = 0;

                    // Fix 2: Flatten let-chains for Stable Rust compatibility
                    if let Some(WaitResult::Message(ConfigItem::Rates(rates_config))) =
                        ctx.config_subscriber.try_next_message()
                    {
                        ctx.rates.set(rates_config);
                    }

                    // Update rc_modes from the rx_frame that has just come in from the radio.
                    ctx.rc_modes.update_activated_modes(&rx_frame);

                    // Note: Ensure this .await does not introduce excessive latency to the UART parser loop
                    ctx.rc_adjustments.process_adjustments(&ctx.config_publisher, &ctx.fast_config_publisher).await;

                    let mut rx_message =
                        RxMessage::new_from(&rx_frame, &ctx.rates, &ctx.rc_modes, loop_count, failsafe);

                    #[cfg(feature = "autopilot")]
                    if let Some(autopilot_message) = ctx.autopilot_receiver.try_changed() {
                        if ctx.rc_modes.is_mode_active(RcMode::ALTITUDE_HOLD) {
                            rx_message.rc_controls.throttle_stick = autopilot_message.rc_controls.throttle_stick;
                        } else if ctx.rc_modes.is_mode_active(RcMode::POSITION_HOLD)
                            || ctx.rc_modes.is_mode_active(RcMode::GPS_RESCUE)
                            || ctx.rc_modes.is_mode_active(RcMode::AUTOPILOT)
                        {
                            rx_message.rc_controls = autopilot_message.rc_controls;
                        }
                    }

                    // Send the rx message to the gyro_pid task.
                    ctx.rx_message_sender.send(rx_message);

                    if loop_count.is_multiple_of(10) {
                        log::info!("              RX:       loop {loop_count}");
                    }
                    loop_count = loop_count.wrapping_add(1);
                }
            }
        }
    }
}

impl RxContext {
    /// Read data from the UART with line-idle/break detection in a target-agnostic way.
    pub async fn read_packet(&mut self) -> Result<usize, ()> {
        #[cfg(any(feature = "rp2040", feature = "rp235xa", feature = "rp235xb"))]
        {
            // embassy_rp returns a Result<usize, ReadToBreakError>
            match self.uart_rx.read_to_break(&mut self.buf).await {
                Ok(n) => Ok(n),
                Err(_) => Err(()), // Map platform errors to a simple generic error
            }
        }

        #[cfg(feature = "stm32")]
        {
            // embassy_stm32 returns a Result<usize, Error> via idle line detection
            match self.uart_rx.read_until_idle(&mut self.buf).await {
                Ok(n) => Ok(n),
                Err(_) => Err(()),
            }
        }
        #[cfg(feature = "host")]
        {
            core::future::ready(()).await;
            Err(())
        }
    }
}
