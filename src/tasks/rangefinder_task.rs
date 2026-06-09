#![cfg(feature = "rangefinder")]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
use log::info;

use crate::sensors::RangefinderData;

const MAX_RANGEFINDER_DATA_SUBSCRIBER_COUNT: usize = 10;
const RANGEFINDER_DATA_PUBLISHER_COUNT: usize = 1;
const RANGEFINDER_DATA_CAPACITY: usize = 1; // only keep the last item

/// `PubSubChannel` for handling `SensorData` updates.
static RANGEFINDER_DATA_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    RangefinderData,
    RANGEFINDER_DATA_CAPACITY,
    MAX_RANGEFINDER_DATA_SUBSCRIBER_COUNT,
    RANGEFINDER_DATA_PUBLISHER_COUNT,
> = PubSubChannel::new();

type RangefinderDataPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    RangefinderData,
    RANGEFINDER_DATA_CAPACITY,
    MAX_RANGEFINDER_DATA_SUBSCRIBER_COUNT,
    RANGEFINDER_DATA_PUBLISHER_COUNT,
>;

pub fn rangefinder_data_publisher<'a>() -> RangefinderDataPublisher<'a> {
    RANGEFINDER_DATA_PUB_SUB_CHANNEL.publisher().expect("rangefinder_data_publisher failed")
}

pub type RangefinderDataSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    RangefinderData,
    RANGEFINDER_DATA_CAPACITY,
    MAX_RANGEFINDER_DATA_SUBSCRIBER_COUNT,
    RANGEFINDER_DATA_PUBLISHER_COUNT,
>;

pub fn rangefinder_data_subscriber<'a>() -> RangefinderDataSubscriber<'a> {
    RANGEFINDER_DATA_PUB_SUB_CHANNEL.subscriber().expect("rangefinder_data_subscriber failed")
}

/// Context for Rangefinder task.
pub struct RangefinderContext<'a> {
    pub rangefinder_data_publisher: RangefinderDataPublisher<'a>,
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
        let rangefinder_data = RangefinderData::default();
        // Publish a message, but if the queue is full, just kick out the oldest message.
        // This may cause some subscribers to miss a message
        ctx.rangefinder_data_publisher.publish_immediate(rangefinder_data);

        if loop_count.is_multiple_of(10) {
            info!("   RANGE:    loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
