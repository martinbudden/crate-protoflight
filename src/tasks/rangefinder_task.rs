#![cfg(feature = "rangefinder")]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
use log::info;

use crate::sensors::RangefinderMessage;

const MAX_RANGEFINDER_SUBSCRIBER_COUNT: usize = 4;
const RANGEFINDER_PUBLISHER_COUNT: usize = 1;
const RANGEFINDER_CAPACITY: usize = 1; // only keep the last item

/// `PubSubChannel` for handling `Rangefinder` updates.
static RANGEFINDER_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    RangefinderMessage,
    RANGEFINDER_CAPACITY,
    MAX_RANGEFINDER_SUBSCRIBER_COUNT,
    RANGEFINDER_PUBLISHER_COUNT,
> = PubSubChannel::new();

type RangefinderPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    RangefinderMessage,
    RANGEFINDER_CAPACITY,
    MAX_RANGEFINDER_SUBSCRIBER_COUNT,
    RANGEFINDER_PUBLISHER_COUNT,
>;

pub fn rangefinder_publisher<'a>() -> RangefinderPublisher<'a> {
    RANGEFINDER_PUB_SUB_CHANNEL.publisher().expect("rangefinder_publisher failed")
}

pub type RangefinderSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    RangefinderMessage,
    RANGEFINDER_CAPACITY,
    MAX_RANGEFINDER_SUBSCRIBER_COUNT,
    RANGEFINDER_PUBLISHER_COUNT,
>;

pub fn rangefinder_subscriber<'a>() -> RangefinderSubscriber<'a> {
    RANGEFINDER_PUB_SUB_CHANNEL.subscriber().expect("rangefinder_subscriber failed")
}

/// Context for Rangefinder task.
pub struct RangefinderContext<'a> {
    pub rangefinder_publisher: RangefinderPublisher<'a>,
}

/// Rangefinder Task Placeholder.
#[embassy_executor::task]
pub async fn rangefinder_task(ctx: &'static mut RangefinderContext<'static>) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(40));
    let mut loop_count: u32 = 0;

    info!("RANGEFINDER: task started");
    loop {
        // Wait for the next tick.
        ticker.next().await;
        let rangefinder_message = RangefinderMessage::default();
        // Publish a message, but if the queue is full, just kick out the oldest message.
        // This may cause some subscribers to miss a message
        ctx.rangefinder_publisher.publish_immediate(rangefinder_message);

        if loop_count.is_multiple_of(10) {
            info!("   RANGE:    loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
