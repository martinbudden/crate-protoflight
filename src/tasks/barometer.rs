#![cfg(feature = "barometer")]

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};
use static_cell::StaticCell;

use crate::{
    barometer_sensors::{Barometer, BarometerDevice, BarometerMessage},
    tasks::errors::TaskInitError,
};

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

pub fn barometer_subscriber() -> Result<BarometerSubscriber, TaskInitError> {
    BAROMETER_PUB_SUB_CHANNEL.subscriber().map_err(|_| TaskInitError::BarometerSubscriberFailed)
}

/// Context for Barometer task.
pub struct BarometerContext {
    pub barometer: Barometer,
    pub barometer_publisher: BarometerPublisher,
}

pub fn init(barometer: Barometer) -> Result<&'static mut BarometerContext, TaskInitError> {
    let ctx = BarometerContext {
        barometer,
        barometer_publisher: BAROMETER_PUB_SUB_CHANNEL
            .publisher()
            .map_err(|_| TaskInitError::BarometerPublisherFailed)?,
    };
    Ok(BAROMETER_CTX.init(ctx))
}

/// Barometer Task Placeholder.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut BarometerContext) {
    // If initialization fails, then we just return.
    log::info!("   BAROMETER: task started");
    let Ok(measurement_interval) = ctx.barometer.init().await else {
        return;
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let mut ticker = embassy_time::Ticker::every(measurement_interval);
    let mut loop_count: u32 = 0;

    log::info!("   BAROMETER: loop started");
    loop {
        // Wait for the next tick.
        ticker.next().await;
        let result = ctx.barometer.make_reading().await;
        if let Ok(()) = result {
            let barometer_message = ctx.barometer.message();
            // Publish a message, but if the queue is full, just kick out the oldest message.
            // This may cause some subscribers to miss a message
            ctx.barometer_publisher.publish_immediate(barometer_message);
        } else {
            log::error!("Barometer make reading failed: {result:?}");
        }

        if loop_count.is_multiple_of(10) {
            log::info!("       BAROMETER:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1);
    }
}
