#![allow(unused)]
use log::info;

use crate::{
    osd::Osd,
    tasks::dispatch::{GyroPidReceiver, SetpointReceiver},
};

pub(crate) static OSD_CTX: static_cell::StaticCell<OsdContext> = static_cell::StaticCell::new();

/// Context for OSD task.
pub struct OsdContext {
    pub gyro_pid_receiver: GyroPidReceiver,
    pub setpoint_receiver: SetpointReceiver,
    pub osd: Osd,
}

/// OSD Task Placeholder.
#[embassy_executor::task]
pub async fn osd_task(ctx: &'static mut OsdContext) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut loop_count: u32 = 0;

    //println!("OSD: Started at 50Hz.");
    info!("      OSD: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;

        // Peek(get) the latest messages without consuming the notifications.
        if let Some(_gyro_pid_message) = ctx.gyro_pid_receiver.try_get() {}
        if let Some(_setpoint_message) = ctx.setpoint_receiver.try_get() {}
        // TODO: subscribe to global_context messages as well.

        // Update the OSD with the latest data.
        ctx.osd.update_display();

        if loop_count.is_multiple_of(10) {
            info!("      OSD:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
