#![cfg(feature = "magnetometer")]

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};
use static_cell::StaticCell;
use vqm::{Matrix3x3f32, Vector3f32};

use crate::{
    magnetometer_sensors::MagnetometerDevice,
    magnetometers::{Magnetometer, MagnetometerMessage},
    tasks::errors::TaskInitError,
};

static MAGNETOMETER_CTX: StaticCell<MagnetometerContext> = StaticCell::new();

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

type MagnetometerPublisher = Publisher<
    'static,
    CriticalSectionRawMutex,
    MagnetometerMessage,
    MAGNETOMETER_PUB_SUB_CAPACITY,
    MAX_MAGNETOMETER_SUBSCRIBER_COUNT,
    MAGNETOMETER_PUBLISHER_COUNT,
>;

pub type MagnetometerSubscriber = Subscriber<
    'static,
    CriticalSectionRawMutex,
    MagnetometerMessage,
    MAGNETOMETER_PUB_SUB_CAPACITY,
    MAX_MAGNETOMETER_SUBSCRIBER_COUNT,
    MAGNETOMETER_PUBLISHER_COUNT,
>;

pub fn magnetometer_subscriber() -> Result<MagnetometerSubscriber, TaskInitError> {
    MAGNETOMETER_PUB_SUB_CHANNEL.subscriber().map_err(|_| TaskInitError::MagnetometerSubscriberFailed)
}

/// Context for Magnetometer task.
pub struct MagnetometerContext {
    magnetometer: Magnetometer,
    magnetometer_publisher: MagnetometerPublisher,
    calibration_matrix: Matrix3x3f32,
}

pub fn init(magnetometer: Magnetometer) -> Result<&'static mut MagnetometerContext, TaskInitError> {
    let ctx = MagnetometerContext {
        magnetometer,
        magnetometer_publisher: MAGNETOMETER_PUB_SUB_CHANNEL
            .publisher()
            .map_err(|_| TaskInitError::MagnetometerPublisherFailed)?,
        calibration_matrix: Matrix3x3f32::identity(),
    };
    Ok(MAGNETOMETER_CTX.init(ctx))
}

/// Magnetometer Task Placeholder.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut MagnetometerContext) {
    // If initialization fails, then we just return.
    log::info!("MAGNETOMETER: task started");
    let Ok(measurement_interval) = ctx.magnetometer.init().await else {
        return;
    };

    let mut ticker = embassy_time::Ticker::every(measurement_interval);
    let mut loop_count: u32 = 0;

    loop {
        // Wait for the next tick.
        ticker.next().await;
        let result = ctx.magnetometer.make_reading().await;
        if let Ok((timestamp, reading)) = result {
            let reading = Vector3f32::new(f32::from(reading[0]), f32::from(reading[1]), f32::from(reading[2]) );
            let reading = ctx.calibration_matrix * reading;
            let magnetometer_message = MagnetometerMessage::new(timestamp, reading);
            // Publish a message, but if the queue is full, just kick out the oldest message.
            // This may cause some subscribers to miss a message
            ctx.magnetometer_publisher.publish_immediate(magnetometer_message);
        } else {
            log::error!("Magnetometer make reading failed: {result:?}");
        }

        if loop_count.is_multiple_of(10) {
            log::info!("    MAG:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1);
    }
}
