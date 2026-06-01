use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    watch::{Receiver, Sender, Watch},
};
use radio_controllers::{RadioControlMessage, Rates, RcModes, RxFrame};

use log::info;

use crate::{
    config::{ConfigItem, ConfigPublisher, ConfigSubscriber, FastConfigPublisher},
    flight::RcAdjustments,
};

const RADIO_WATCH_COUNT: usize = 1;
static RADIO_WATCH: Watch<CriticalSectionRawMutex, RadioControlMessage, RADIO_WATCH_COUNT> = Watch::new();

pub type RadioSender = Sender<'static, CriticalSectionRawMutex, RadioControlMessage, RADIO_WATCH_COUNT>;
pub fn radio_sender() -> RadioSender {
    RADIO_WATCH.sender()
}

pub type RadioReceiver = Receiver<'static, CriticalSectionRawMutex, RadioControlMessage, RADIO_WATCH_COUNT>;
pub fn radio_receiver() -> RadioReceiver {
    RADIO_WATCH.receiver().expect("radio receiver failed")
}

#[cfg(feature = "autopilot")]
use crate::tasks::autopilot_task::AutopilotReceiver;

/// Context for `radio_task`.
pub struct RadioContext<'a> {
    pub radio_sender: RadioSender,
    pub config_subscriber: ConfigSubscriber<'a>,
    pub config_publisher: ConfigPublisher<'a>,
    pub fast_config_publisher: FastConfigPublisher<'a>,
    #[cfg(feature = "autopilot")]
    pub autopilot_receiver: AutopilotReceiver,
    pub rc_modes: RcModes,
    pub rates: Rates,
    pub rc_adjustments: RcAdjustments,
}

#[embassy_executor::task]
pub async fn radio_task(ctx: &'static mut RadioContext<'static>) {
    // 50Hz = 20ms interval
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(20));
    let mut loop_count: u32 = 0;

    info!("    RADIO: task started");

    loop {
        // TODO: rx_frame should be obtained on an interrupt form the radio receiver (UART).
        // For now we just wait for the next tick and create a dummy rx_frame.
        ticker.next().await;
        let rx_frame = RxFrame::new();
        // TODO: we need to do some failsafe checking here.
        let failsafe = 0;

        // check if there has been in-flight adjustment of the rates, if so apply them.
        if let Some(wait_result) = ctx.config_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(ConfigItem::Rates(rates_config)) = wait_result
        {
            ctx.rates.set(rates_config);
        }

        #[cfg(feature = "autopilot")]
        if let Some(_autopilot_message) = ctx.autopilot_receiver.try_changed() {
            // TODO: if there is a message from the autopilot, then act on it.
        }

        // Update rc_modes from the rx_frame that has just come in from the radio.
        ctx.rc_modes.update_activated_modes(&rx_frame);
        ctx.rc_adjustments.process_adjustments(&ctx.config_publisher, &ctx.fast_config_publisher).await;

        let radio_control_message =
            RadioControlMessage::new_from(&rx_frame, &ctx.rates, &ctx.rc_modes, loop_count, failsafe);

        // Send the radio control message. This will be picked by the gyro_pid task.
        ctx.radio_sender.send(radio_control_message);

        if loop_count.is_multiple_of(5) {
            info!("    RADIO:    loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
