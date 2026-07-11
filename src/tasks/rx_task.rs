use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    watch::{Receiver, Sender, Watch},
};

#[cfg(feature = "autopilot")]
use radio_controllers::RcMode;
use radio_controllers::{Rates, RcModes, RxFrame};

use crate::{
    config::{ConfigItem, ConfigPublisher, ConfigSubscriber, FastConfigPublisher},
    flight::{RcAdjustments, RxMessage},
};

// Note, we use a `Watch` rather than a `Signal` since the receiver (`gyro_pid_task`) uses `try_changed` to see if the value has changed.
const RX_WATCH_COUNT: usize = 2;
static RX_WATCH: Watch<CriticalSectionRawMutex, RxMessage, RX_WATCH_COUNT> = Watch::new();

type RxSender = Sender<'static, CriticalSectionRawMutex, RxMessage, RX_WATCH_COUNT>;
pub fn rx_sender() -> RxSender {
    RX_WATCH.sender()
}

pub type RxReceiver = Receiver<'static, CriticalSectionRawMutex, RxMessage, RX_WATCH_COUNT>;

#[allow(clippy::expect_used)]
pub fn rx_receiver() -> RxReceiver {
    RX_WATCH.receiver().expect("rx_receiver failed")
}

#[cfg(feature = "autopilot")]
use crate::tasks::autopilot_task::AutopilotReceiver;

/// Context for the `flight_control_task`.
pub struct RxContext<'a> {
    pub rx_sender: RxSender,
    pub config_subscriber: ConfigSubscriber<'a>,
    /// To publish in-flight adjustments.
    pub config_publisher: ConfigPublisher<'a>,
    /// To publish in-flight adjustments.
    pub fast_config_publisher: FastConfigPublisher<'a>,
    #[cfg(feature = "autopilot")]
    pub autopilot_receiver: AutopilotReceiver,
    pub rc_modes: RcModes,
    pub rates: Rates,
    pub rc_adjustments: RcAdjustments,
}

/// The rx task waits (with a timeout) for a packet from the radio and when one arrives it:
/// 1. Checks for any in-flight adjustments of rates.
/// 2. Updates the control modes using the AUX channel values.
/// 3. Creates a `FlightControl` message from the values in the radio packet.
/// 4. Checks if a `FlightControl` message has arrived from the Autopilot, and processes it.
/// 5. Sends the `FlightControl` message to the `gyro_pid` task.
/// If the timeout expires, then failsafe handling is invoked.
#[embassy_executor::task]
pub async fn rx_task(ctx: &'static mut RxContext<'static>) {
    let mut loop_count: u32 = 0;
    // 50Hz = 20ms interval
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(20));

    log::info!("       RX: task started");

    loop {
        // TODO: rx_frame should be obtained on an interrupt form the radio receiver (UART).
        // TODO: we need to do some failsafe checking here.
        // For now we just wait for the next tick and create a dummy rx_frame.
        ticker.next().await;
        let rx_frame = RxFrame::new();
        let failsafe = 0;

        // check if there has been in-flight adjustment of the rates, if so apply them.
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
            // TODO: if there is a message from the autopilot, then act on it.
            if ctx.rc_modes.is_mode_active(RcMode::ALTITUDE_HOLD) {
                rx_message.controls.throttle_stick = autopilot_message.controls.throttle_stick;
            } else if ctx.rc_modes.is_mode_active(RcMode::POSITION_HOLD)
                || ctx.rc_modes.is_mode_active(RcMode::GPS_RESCUE)
                || ctx.rc_modes.is_mode_active(RcMode::AUTOPILOT)
            {
                rx_message.controls = autopilot_message.controls;
            }
        }

        // Send the rx message. This will be picked by the gyro_pid task.
        ctx.rx_sender.send(rx_message);

        if loop_count.is_multiple_of(5) {
            log::info!("            RX:       loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
