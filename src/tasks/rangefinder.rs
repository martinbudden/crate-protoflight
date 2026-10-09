#![cfg(feature = "rangefinder")]

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};
use static_cell::StaticCell;

use crate::{
    rangefinder_sensors::{Rangefinder, RangefinderDevice, RangefinderMessage},
    tasks::errors::TaskInitError,
};

const MAX_RANGEFINDER_SUBSCRIBER_COUNT: usize = 4;
const RANGEFINDER_PUBLISHER_COUNT: usize = 1;
const RANGEFINDER_PUB_SUB_CAPACITY: usize = 1; // only keep the last item

/// `PubSubChannel` for handling `Rangefinder` updates.
static RANGEFINDER_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    RangefinderMessage,
    RANGEFINDER_PUB_SUB_CAPACITY,
    MAX_RANGEFINDER_SUBSCRIBER_COUNT,
    RANGEFINDER_PUBLISHER_COUNT,
> = PubSubChannel::new();

type RangefinderPublisher = Publisher<
    'static,
    CriticalSectionRawMutex,
    RangefinderMessage,
    RANGEFINDER_PUB_SUB_CAPACITY,
    MAX_RANGEFINDER_SUBSCRIBER_COUNT,
    RANGEFINDER_PUBLISHER_COUNT,
>;

pub type RangefinderSubscriber = Subscriber<
    'static,
    CriticalSectionRawMutex,
    RangefinderMessage,
    RANGEFINDER_PUB_SUB_CAPACITY,
    MAX_RANGEFINDER_SUBSCRIBER_COUNT,
    RANGEFINDER_PUBLISHER_COUNT,
>;

pub fn rangefinder_subscriber() -> Result<RangefinderSubscriber, TaskInitError> {
    RANGEFINDER_PUB_SUB_CHANNEL.subscriber().map_err(|_| TaskInitError::RangefinderSubscriberFailed)
}

static RANGEFINDER_CTX: StaticCell<RangefinderContext> = StaticCell::new();

/// Context for Rangefinder task.
pub struct RangefinderContext {
    pub rangefinder: Rangefinder,
    pub rangefinder_publisher: RangefinderPublisher,
}

pub fn init(rangefinder: Rangefinder) -> Result<&'static mut RangefinderContext, TaskInitError> {
    let ctx = RangefinderContext {
        rangefinder,
        rangefinder_publisher: RANGEFINDER_PUB_SUB_CHANNEL
            .publisher()
            .map_err(|_| TaskInitError::RangefinderPublisherFailed)?,
    };
    Ok(RANGEFINDER_CTX.init(ctx))
}

/// Rangefinder Task Placeholder.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut RangefinderContext) {
    const TASK_FREQUENCY_HZ: u64 = 40;
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(TASK_FREQUENCY_HZ));
    let mut loop_count: u32 = 0;

    log::info!(" RANGEFINDER: task started");
    loop {
        // Wait for the next tick.
        ticker.next().await;
        let rangefinder_message = ctx.rangefinder.message();
        // Publish a message, but if the queue is full, just kick out the oldest message.
        // This may cause some subscribers to miss a message
        ctx.rangefinder_publisher.publish_immediate(rangefinder_message);

        if loop_count.is_multiple_of(10) {
            log::info!("     RANGE:    loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1);
    }
}
