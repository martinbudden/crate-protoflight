#![allow(unused)]
/// Barometer Task Placeholder.
///
use embassy_time::Duration;

use log::info;
use static_cell::StaticCell;

pub(crate) static BAROMETER_CTX: StaticCell<BarometerContext> = StaticCell::new();

/// Context for Barometer task.
pub struct BarometerContext {}

#[embassy_executor::task]
pub async fn barometer_task(ctx: &'static mut BarometerContext) {
    let mut ticker = embassy_time::Ticker::every(Duration::from_hz(40));
    let mut loop_count: u32 = 0;

    info!("BAROMETER: task started");
    loop {
        // Wait for the next tick.
        ticker.next().await;

        if loop_count.is_multiple_of(10) {
            info!("BAROMETER:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
