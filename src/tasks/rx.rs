use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    watch::{Receiver, Sender, Watch},
};

#[cfg(feature = "autopilot")]
use radio_controllers::RcMode;
use radio_controllers::{Radio, Rates, RatesConfig, RcModes, RxChannel, RxRadio};
use static_cell::StaticCell;

use crate::{
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
use crate::tasks::autopilot::{AutopilotReceiver, autopilot_receiver};

/// Context for the receiver task.
pub struct RxContext {
    pub radio: Radio,
    pub rx_message_sender: RxMessageSender,
    pub config_subscriber: ConfigSubscriber,
    /// To publish in-flight adjustments.
    pub config_publisher: ConfigPublisher,
    /// To publish in-flight adjustments.
    pub fast_config_publisher: FastConfigPublisher,
    pub rc_modes: RcModes,
    pub rates: Rates,
    pub rc_adjustments: RcAdjustments,
    #[cfg(feature = "autopilot")]
    pub autopilot_receiver: AutopilotReceiver,
}

impl RxContext {
    #[rustfmt::skip]
    pub fn new(
        radio: Radio,
        rates_config: RatesConfig,
    ) -> Self {
        Self {
            radio,
            rx_message_sender:rx_message_sender(),
            config_subscriber:config_subscriber(),
            config_publisher:config_publisher(),
            fast_config_publisher:fast_config_publisher(),
            rates: Rates::new(rates_config),
            rc_modes: RcModes::with_mac_arm(),
            rc_adjustments: RcAdjustments::new(),
            #[cfg(feature = "autopilot")] autopilot_receiver:autopilot_receiver(),
        }
    }
}

pub fn init(radio: Radio, rates: RatesConfig) -> &'static mut RxContext {
    RX_CTX.init(RxContext::new(radio, rates))
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
    // 50Hz = 20ms interval
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(20));

    log::info!("          RX: task started");

    loop {
        // TODO: rx_frame should be obtained on an interrupt form the radio receiver (UART). For now we just wait for the next tick.
        ticker.next().await;
        let mut rx_frame = ctx.radio.rx_frame();

        // Simulate the user changing the arming channel.
        rx_frame.channels[RxChannel::AUX1] = match loop_count {
            0..100 | 200..400 => RxChannel::MID_HIGH,
            _ => RxChannel::LOW,
        };

        // TODO: we need to do some failsafe checking here.
        let failsafe = 0;

        // check if there has been in-flight adjustment of the rates, if so apply them. This is a very infrequent event.
        if let Some(wait_result) = ctx.config_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(ConfigItem::Rates(rates_config)) = wait_result
        {
            ctx.rates.set(rates_config);
        }

        // Update rc_modes from the rx_frame that has just come in from the radio.
        ctx.rc_modes.update_activated_modes(&rx_frame);

        ctx.rc_adjustments.process_adjustments(&ctx.config_publisher, &ctx.fast_config_publisher).await;

        #[allow(unused_mut)]
        let mut rx_message = RxMessage::new_from(&rx_frame, &ctx.rates, &ctx.rc_modes, loop_count, failsafe);

        #[cfg(feature = "autopilot")]
        if let Some(autopilot_message) = ctx.autopilot_receiver.try_changed() {
            // If there is a message from the autopilot, use it to set the controls.
            if ctx.rc_modes.is_mode_active(RcMode::ALTITUDE_HOLD) {
                rx_message.rc_controls.throttle_stick = autopilot_message.rc_controls.throttle_stick;
            } else if ctx.rc_modes.is_mode_active(RcMode::POSITION_HOLD)
                || ctx.rc_modes.is_mode_active(RcMode::GPS_RESCUE)
                || ctx.rc_modes.is_mode_active(RcMode::AUTOPILOT)
            {
                rx_message.rc_controls = autopilot_message.rc_controls;
            }
        }

        // Send the rx message. This will be picked by the gyro_pid task.
        ctx.rx_message_sender.send(rx_message);

        if loop_count.is_multiple_of(10) {
            log::info!("            RX:       loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
