use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use radio_controllers::{RadioControlMessage, Rates, RcModes, RxFrame};

use log::info;
use static_cell::StaticCell;

use crate::{
    config::{CONFIG_PUB_SUB_CHANNEL, ConfigItem, ConfigSubscriber, GYRO_PID_PUB_SUB_CHANNEL},
    flight::RcAdjustments,
};

pub(crate) static RADIO_CTX: StaticCell<RadioContext> = StaticCell::new();

const RADIO_WATCH_COUNT: usize = 1;
static RADIO_WATCH: Watch<CriticalSectionRawMutex, RadioControlMessage, RADIO_WATCH_COUNT> = Watch::new();

pub type RadioSender =
    embassy_sync::watch::Sender<'static, CriticalSectionRawMutex, RadioControlMessage, RADIO_WATCH_COUNT>;
pub fn radio_sender() -> RadioSender {
    RADIO_WATCH.sender()
}

pub type RadioReceiver =
    embassy_sync::watch::Receiver<'static, CriticalSectionRawMutex, RadioControlMessage, RADIO_WATCH_COUNT>;
pub fn radio_receiver() -> RadioReceiver {
    RADIO_WATCH.receiver().expect("radio receiver failed")
}

const AUTOPILOT_WATCH_COUNT: usize = 1;
static AUTOPILOT_WATCH: Watch<CriticalSectionRawMutex, RadioControlMessage, AUTOPILOT_WATCH_COUNT> = Watch::new();

pub type AutopilotSender =
    embassy_sync::watch::Sender<'static, CriticalSectionRawMutex, RadioControlMessage, AUTOPILOT_WATCH_COUNT>;
pub fn autopilot_sender() -> AutopilotSender {
    AUTOPILOT_WATCH.sender()
}

pub type AutopilotReceiver =
    embassy_sync::watch::Receiver<'static, CriticalSectionRawMutex, RadioControlMessage, AUTOPILOT_WATCH_COUNT>;
pub fn autopilot_receiver() -> AutopilotReceiver {
    AUTOPILOT_WATCH.receiver().expect("radio receiver failed")
}

/// Context for radio_task.
pub struct RadioContext<'a> {
    pub radio_sender: RadioSender,
    pub _autopilot_receiver: AutopilotReceiver,
    pub config_subscriber: ConfigSubscriber<'a>,
    pub rc_modes: RcModes,
    pub rates: Rates,
    pub rc_adjustments: RcAdjustments,
}

#[embassy_executor::task]
pub async fn radio_task(ctx: &'static mut RadioContext<'static>) {
    let config_publisher = CONFIG_PUB_SUB_CHANNEL.publisher().expect("failed to create Radio config publisher");
    let gyro_pid_publisher = GYRO_PID_PUB_SUB_CHANNEL.publisher().expect("failed to create Radio gyro_pid publisher");
    // 50Hz = 20ms interval
    let mut ticker = embassy_time::Ticker::every(Duration::from_millis(20));
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
        while let Some(wait_result) = ctx.config_subscriber.try_next_message() {
            if let embassy_sync::pubsub::WaitResult::Message(ConfigItem::Rates(config)) = wait_result {
                ctx.rates.set(config);
            }
        }

        // Update rc_modes from the rx_frame that has just come in from the radio.
        ctx.rc_modes.update_activated_modes(&rx_frame);
        ctx.rc_adjustments.process_adjustments(&config_publisher, &gyro_pid_publisher).await;

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
