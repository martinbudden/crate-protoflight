#![cfg(feature = "barometer")]

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};
use static_cell::StaticCell;

use crate::barometer_sensors::{Barometer, BarometerDevice, BarometerMessage};

static BAROMETER_CTX: StaticCell<BarometerContext> = StaticCell::new();

const MAX_BAROMETER_SUBSCRIBER_COUNT: usize = 4;
const BAROMETER_PUBLISHER_COUNT: usize = 1;
const BAROMETER_PUB_SUB_CAPACITY: usize = 1; // only keep the last item

/// `PubSubChannel` for handling `barometer` updates.
static BAROMETER_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    BarometerMessage,
    BAROMETER_PUB_SUB_CAPACITY,
    MAX_BAROMETER_SUBSCRIBER_COUNT,
    BAROMETER_PUBLISHER_COUNT,
> = PubSubChannel::new();

type BarometerPublisher = Publisher<
    'static,
    CriticalSectionRawMutex,
    BarometerMessage,
    BAROMETER_PUB_SUB_CAPACITY,
    MAX_BAROMETER_SUBSCRIBER_COUNT,
    BAROMETER_PUBLISHER_COUNT,
>;

pub type BarometerSubscriber = Subscriber<
    'static,
    CriticalSectionRawMutex,
    BarometerMessage,
    BAROMETER_PUB_SUB_CAPACITY,
    MAX_BAROMETER_SUBSCRIBER_COUNT,
    BAROMETER_PUBLISHER_COUNT,
>;

#[allow(clippy::expect_used)]
pub fn barometer_subscriber() -> BarometerSubscriber {
    BAROMETER_PUB_SUB_CHANNEL.subscriber().expect("barometer_subscriber failed")
}

/// Context for Barometer task.
pub struct BarometerContext {
    pub barometer: Barometer,
    pub barometer_publisher: BarometerPublisher,
}

impl BarometerContext {
    pub fn new(barometer: Barometer) -> Self {
        #[allow(clippy::expect_used)]
        Self {
            barometer,
            barometer_publisher: BAROMETER_PUB_SUB_CHANNEL.publisher().expect("barometer_publisher failed"),
        }
    }
}

pub fn init(barometer: Barometer) -> &'static mut BarometerContext {
    BAROMETER_CTX.init(BarometerContext::new(barometer))
}

/// Barometer Task Placeholder.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut BarometerContext) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(40));
    let mut loop_count: u32 = 0;

    log::info!("   BAROMETER: task started");
    loop {
        // Wait for the next tick.
        ticker.next().await;
        let barometer_message = ctx.barometer.message();
        // Publish a message, but if the queue is full, just kick out the oldest message.
        // This may cause some subscribers to miss a message
        ctx.barometer_publisher.publish_immediate(barometer_message);

        if loop_count.is_multiple_of(10) {
            log::info!("     BAROMETER:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
