#![cfg(feature = "barometer")]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
use log::info;

use crate::sensors::BarometerData;

const MAX_BAROMETER_DATA_SUBSCRIBER_COUNT: usize = 10;
const BAROMETER_DATA_PUBLISHER_COUNT: usize = 1;
const BAROMETER_DATA_CAPACITY: usize = 1; // only keep the last item

/// `PubSubChannel` for handling `SensorData` updates.
static BAROMETER_DATA_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    BarometerData,
    BAROMETER_DATA_CAPACITY,
    MAX_BAROMETER_DATA_SUBSCRIBER_COUNT,
    BAROMETER_DATA_PUBLISHER_COUNT,
> = PubSubChannel::new();

type BarometerDataPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    BarometerData,
    BAROMETER_DATA_CAPACITY,
    MAX_BAROMETER_DATA_SUBSCRIBER_COUNT,
    BAROMETER_DATA_PUBLISHER_COUNT,
>;

pub fn barometer_data_publisher<'a>() -> BarometerDataPublisher<'a> {
    BAROMETER_DATA_PUB_SUB_CHANNEL.publisher().expect("barometer_data_publisher failed")
}

pub type BarometerDataSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    BarometerData,
    BAROMETER_DATA_CAPACITY,
    MAX_BAROMETER_DATA_SUBSCRIBER_COUNT,
    BAROMETER_DATA_PUBLISHER_COUNT,
>;

pub fn barometer_data_subscriber<'a>() -> BarometerDataSubscriber<'a> {
    BAROMETER_DATA_PUB_SUB_CHANNEL.subscriber().expect("barometer_data_subscriber failed")
}

/// Context for Barometer task.
pub struct BarometerContext<'a> {
    pub barometer_data_publisher: BarometerDataPublisher<'a>,
}

/// Barometer Task Placeholder.
#[embassy_executor::task]
pub async fn barometer_task(ctx: &'static mut BarometerContext<'static>) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(40));
    let mut loop_count: u32 = 0;

    info!("BAROMETER: task started");
    loop {
        // Wait for the next tick.
        ticker.next().await;
        let barometer_data = BarometerData::default();
        // Publish a message, but if the queue is full, just kick out the oldest message.
        // This may cause some subscribers to miss a message
        ctx.barometer_data_publisher.publish_immediate(barometer_data);

        if loop_count.is_multiple_of(10) {
            info!("     BAROMETER:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
