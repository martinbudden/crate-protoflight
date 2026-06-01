#![allow(unused)]

use log::info;

use crate::{
    sensor_data::{SensorDataItem, SensorDataPublisher},
    sensors::BarometerData,
};

pub(crate) static BAROMETER_CTX: static_cell::StaticCell<BarometerContext> = static_cell::StaticCell::new();

/// Context for Barometer task.
pub struct BarometerContext<'a> {
    pub sensor_data_publisher: SensorDataPublisher<'a>,
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
        ctx.sensor_data_publisher.publish_immediate(SensorDataItem::Barometer(barometer_data));

        if loop_count.is_multiple_of(10) {
            info!("BAROMETER:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
