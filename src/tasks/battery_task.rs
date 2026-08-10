#![cfg(feature = "battery")]

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};

use crate::sensors::BatteryMessage;

const MAX_BATTERY_SUBSCRIBER_COUNT: usize = 4;
const BATTERY_PUBLISHER_COUNT: usize = 1;
const BATTERY_PUB_SUB_CAPACITY: usize = 1; // only keep the last item

/// `PubSubChannel` for handling `battery` updates.
static BATTERY_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    BatteryMessage,
    BATTERY_PUB_SUB_CAPACITY,
    MAX_BATTERY_SUBSCRIBER_COUNT,
    BATTERY_PUBLISHER_COUNT,
> = PubSubChannel::new();

type BatteryPublisher = Publisher<
    'static,
    CriticalSectionRawMutex,
    BatteryMessage,
    BATTERY_PUB_SUB_CAPACITY,
    MAX_BATTERY_SUBSCRIBER_COUNT,
    BATTERY_PUBLISHER_COUNT,
>;

#[allow(unused)]
pub type BatterySubscriber = Subscriber<
    'static,
    CriticalSectionRawMutex,
    BatteryMessage,
    BATTERY_PUB_SUB_CAPACITY,
    MAX_BATTERY_SUBSCRIBER_COUNT,
    BATTERY_PUBLISHER_COUNT,
>;

#[allow(clippy::expect_used)]
#[allow(unused)]
pub fn battery_subscriber() -> BatterySubscriber {
    BATTERY_PUB_SUB_CHANNEL.subscriber().expect("battery_subscriber failed")
}

/// Context for Battery task.
pub struct BatteryContext {
    pub battery_publisher: BatteryPublisher,
}

impl BatteryContext {
    pub fn new() -> Self {
        #[allow(clippy::expect_used)]
        Self { battery_publisher: BATTERY_PUB_SUB_CHANNEL.publisher().expect("battery_publisher failed") }
    }
}

/// Battery Task Placeholder.
///

#[embassy_executor::task]
pub async fn battery_task(ctx: &'static mut BatteryContext) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut loop_count: u32 = 0;

    log::info!("     BATTERY: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;

        // TODO: get the battery data by reading the battery.
        let battery_message = BatteryMessage::default();
        ctx.battery_publisher.publish_immediate(battery_message);

        if loop_count.is_multiple_of(10) {
            log::info!("       BATTERY:  loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
