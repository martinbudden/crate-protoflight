#![allow(unused)]

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};
use radio_controllers::{FailsafeConfig, RxFrame, RxLinkStatus};
use static_cell::StaticCell;

use crate::{
    flight::RxMessage,
    tasks::rx::{RxMessageReceiver, rx_message_receiver},
};

static FAILSAFE_CTX: StaticCell<FailsafeContext> = StaticCell::new();

// at least the rx and autopilot are subscribers, possibly also OSD.
const MAX_FAILSAFE_SUBSCRIBER_COUNT: usize = 3;
const FAILSAFE_PUBLISHER_COUNT: usize = 1;
const FAILSAFE_PUB_SUB_CAPACITY: usize = 1; // only keep the last item

/// `PubSubChannel` for handling `failsafe` updates.
static FAILSAFE_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    FailsafeMessage,
    FAILSAFE_PUB_SUB_CAPACITY,
    MAX_FAILSAFE_SUBSCRIBER_COUNT,
    FAILSAFE_PUBLISHER_COUNT,
> = PubSubChannel::new();

type FailsafePublisher = Publisher<
    'static,
    CriticalSectionRawMutex,
    FailsafeMessage,
    FAILSAFE_PUB_SUB_CAPACITY,
    MAX_FAILSAFE_SUBSCRIBER_COUNT,
    FAILSAFE_PUBLISHER_COUNT,
>;

#[allow(unused)]
pub type FailsafeSubscriber = Subscriber<
    'static,
    CriticalSectionRawMutex,
    FailsafeMessage,
    FAILSAFE_PUB_SUB_CAPACITY,
    MAX_FAILSAFE_SUBSCRIBER_COUNT,
    FAILSAFE_PUBLISHER_COUNT,
>;

#[allow(clippy::expect_used)]
#[allow(unused)]
pub fn failsafe_subscriber() -> FailsafeSubscriber {
    FAILSAFE_PUB_SUB_CHANNEL.subscriber().expect("failsafe_subscriber failed")
}

/// Context for Failsafe task.
pub struct FailsafeContext {
    failsafe_publisher: FailsafePublisher,
    failsafe_handler: FailsafeHandler,
    rx_message: RxMessage,
    rx_message_receiver: RxMessageReceiver,
}

impl FailsafeContext {
    pub fn new(config: &FailsafeConfig) -> Self {
        Self {
            #[allow(clippy::expect_used)]
            failsafe_publisher: FAILSAFE_PUB_SUB_CHANNEL.publisher().expect("failsafe_publisher failed"),
            failsafe_handler: FailsafeHandler::new(config),
            rx_message: RxMessage::new(),
            rx_message_receiver: rx_message_receiver(),
        }
    }
}

pub fn init(config: &FailsafeConfig) -> &'static mut FailsafeContext {
    FAILSAFE_CTX.init(FailsafeContext::new(config))
}

/// Failsafe Task.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut FailsafeContext) {
    const TASK_FREQUENCY_HZ: u64 = 10;
    _ = ctx;
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(TASK_FREQUENCY_HZ));
    let mut tick_count: u32 = 0;

    log::info!("    FAILSAFE: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;
        tick_count = tick_count.wrapping_add(1);

        if let Some(rx_message) = ctx.rx_message_receiver.try_changed() {
            ctx.rx_message = rx_message;
        }

        ctx.failsafe_handler.on_tick(&ctx.rx_message, tick_count);
        let failsafe_message = FailsafeMessage { state: ctx.failsafe_handler.state };
        ctx.failsafe_publisher.publish_immediate(failsafe_message);

        if tick_count.is_multiple_of(10) {
            log::info!("        FAILSAFE:  loop {tick_count}");
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FailsafeMessage {
    pub state: FailsafeState,
}

impl Default for FailsafeMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl FailsafeMessage {
    pub fn new() -> Self {
        Self { state: FailsafeState::default() }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FailsafeState {
    #[default]
    Idle,
    RxLossMonitoring,
    RxLossRecovered,
    Landing,
    Landed,
    GpsRescue,
}

pub struct FailsafeHandler {
    state: FailsafeState,
    rx_receiver: RxMessageReceiver,
    loss_detected_at: u32,
}

impl FailsafeHandler {
    pub fn new(config: &FailsafeConfig) -> Self {
        Self { state: FailsafeState::Idle, rx_receiver: rx_message_receiver(), loss_detected_at: 0 }
    }
}

impl FailsafeHandler {
    pub fn on_tick(&mut self, rx_message: &RxMessage, tick_count: u32) {
        let link_status = rx_message.rc_controls.link_status;
        self.state = match core::mem::take(&mut self.state) {
            FailsafeState::Idle => {
                if link_status == RxLinkStatus::Ok {
                    FailsafeState::Idle
                } else {
                    self.loss_detected_at = tick_count;
                    FailsafeState::RxLossMonitoring
                }
            }
            FailsafeState::RxLossMonitoring => {
                if link_status == RxLinkStatus::Ok {
                    FailsafeState::RxLossRecovered
                } else if tick_count.wrapping_sub(self.loss_detected_at) <= 10 {
                    FailsafeState::RxLossMonitoring
                } else {
                    FailsafeState::Landing
                }
            }
            FailsafeState::RxLossRecovered => {
                if link_status == RxLinkStatus::Ok {
                    FailsafeState::Idle
                } else {
                    FailsafeState::RxLossMonitoring
                }
            }
            FailsafeState::Landing => {
                if link_status == RxLinkStatus::Ok {
                    FailsafeState::Idle
                } else {
                    FailsafeState::Landing
                }
            }
            FailsafeState::Landed => {
                if link_status == RxLinkStatus::Ok {
                    FailsafeState::Idle
                } else {
                    FailsafeState::Landed
                }
            }
            FailsafeState::GpsRescue => {
                if link_status == RxLinkStatus::Ok {
                    FailsafeState::Idle
                } else {
                    FailsafeState::GpsRescue
                }
            }
        };
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<FailsafeState>();
        is_full::<FailsafeMessage>();
    }
}
