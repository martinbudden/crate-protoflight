#![cfg(feature = "battery")]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
use log::info;

use crate::sensors::BatteryState;

const MAX_BATTERY_DATA_SUBSCRIBER_COUNT: usize = 10;
const BATTERY_DATA_PUBLISHER_COUNT: usize = 1;
const BATTERY_DATA_CAPACITY: usize = 1; // only keep the last item

/// `PubSubChannel` for handling `SensorData` updates.
static BATTERY_DATA_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    BatteryState,
    BATTERY_DATA_CAPACITY,
    MAX_BATTERY_DATA_SUBSCRIBER_COUNT,
    BATTERY_DATA_PUBLISHER_COUNT,
> = PubSubChannel::new();

type BatteryDataPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    BatteryState,
    BATTERY_DATA_CAPACITY,
    MAX_BATTERY_DATA_SUBSCRIBER_COUNT,
    BATTERY_DATA_PUBLISHER_COUNT,
>;

pub fn battery_data_publisher<'a>() -> BatteryDataPublisher<'a> {
    BATTERY_DATA_PUB_SUB_CHANNEL.publisher().expect("battery_data_publisher failed")
}

#[allow(unused)]
pub type BatteryDataSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    BatteryState,
    BATTERY_DATA_CAPACITY,
    MAX_BATTERY_DATA_SUBSCRIBER_COUNT,
    BATTERY_DATA_PUBLISHER_COUNT,
>;

#[allow(unused)]
pub fn battery_data_subscriber<'a>() -> BatteryDataSubscriber<'a> {
    BATTERY_DATA_PUB_SUB_CHANNEL.subscriber().expect("battery_data_subscriber failed")
}

/// Context for Battery task.
pub struct BatteryContext<'a> {
    pub battery_data_publisher: BatteryDataPublisher<'a>,
}

/// Battery Task Placeholder.
///

#[embassy_executor::task]
pub async fn battery_task(ctx: &'static mut BatteryContext<'static>) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut loop_count: u32 = 0;

    info!("  BATTERY: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;

        let battery_data = BatteryState::default();
        ctx.battery_data_publisher.publish_immediate(battery_data);

        if loop_count.is_multiple_of(10) {
            info!("       BATTERY:  loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
