#![cfg(feature = "magnetometer")]

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};

use crate::sensors::MagnetometerMessage;

const MAX_MAGNETOMETER_SUBSCRIBER_COUNT: usize = 4;
const MAGNETOMETER_PUBLISHER_COUNT: usize = 1;
const MAGNETOMETER_PUB_SUB_CAPACITY: usize = 1; // only keep the last item

/// `PubSubChannel` for handling `Magnetometer` updates.
static MAGNETOMETER_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    MagnetometerMessage,
    MAGNETOMETER_PUB_SUB_CAPACITY,
    MAX_MAGNETOMETER_SUBSCRIBER_COUNT,
    MAGNETOMETER_PUBLISHER_COUNT,
> = PubSubChannel::new();

type MagnetometerPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    MagnetometerMessage,
    MAGNETOMETER_PUB_SUB_CAPACITY,
    MAX_MAGNETOMETER_SUBSCRIBER_COUNT,
    MAGNETOMETER_PUBLISHER_COUNT,
>;

#[allow(clippy::expect_used)]
pub fn magnetometer_publisher<'a>() -> MagnetometerPublisher<'a> {
    MAGNETOMETER_PUB_SUB_CHANNEL.publisher().expect("magnetometer_publisher failed")
}

pub type MagnetometerSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    MagnetometerMessage,
    MAGNETOMETER_PUB_SUB_CAPACITY,
    MAX_MAGNETOMETER_SUBSCRIBER_COUNT,
    MAGNETOMETER_PUBLISHER_COUNT,
>;

#[allow(clippy::expect_used)]
pub fn magnetometer_subscriber<'a>() -> MagnetometerSubscriber<'a> {
    MAGNETOMETER_PUB_SUB_CHANNEL.subscriber().expect("magnetometer_subscriber failed")
}

/// Context for Magnetometer task.
pub struct MagnetometerContext<'a> {
    pub magnetometer_publisher: MagnetometerPublisher<'a>,
}

impl<'a> MagnetometerContext<'a> {
    pub fn new(magnetometer_publisher: MagnetometerPublisher<'a>) -> Self {
        Self { magnetometer_publisher }
    }
}

/// Magnetometer Task Placeholder.
#[embassy_executor::task]
pub async fn magnetometer_task(ctx: &'static mut MagnetometerContext<'static>) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(40));
    let mut loop_count: u32 = 0;

    log::info!("MAGNETOMETER: task started");
    loop {
        // Wait for the next tick.
        ticker.next().await;
        let magnetometer_message = MagnetometerMessage::default();
        // Publish a message, but if the queue is full, just kick out the oldest message.
        // This may cause some subscribers to miss a message
        ctx.magnetometer_publisher.publish_immediate(magnetometer_message);

        if loop_count.is_multiple_of(10) {
            log::info!("     MAG:    loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
