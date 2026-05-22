#![allow(unused)]
/// OSD Placeholder.
///
use embassy_time::Duration;

use log::info;
use static_cell::StaticCell;

use crate::dispatch::{GyroPidReceiver, SetpointReceiver};
use crate::osd::Osd;
pub(crate) static OSD_CTX: StaticCell<OsdContext> = StaticCell::new();

/// Context for OSD task.
pub struct OsdContext {
    pub gyro_pid_receiver: GyroPidReceiver,
    pub setpoint_receiver: SetpointReceiver,
    pub osd: Osd,
}

#[embassy_executor::task]
pub async fn osd_task(ctx: &'static mut OsdContext) {
    // 50Hz = 20ms interval
    let mut ticker = embassy_time::Ticker::every(Duration::from_millis(200));
    let mut loop_count: u32 = 0;

    //println!("OSD: Started at 50Hz.");
    info!("      OSD: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;

        // Peek(get) the latest messages without consuming the notifications.
        let _gyro_pid_message = ctx.gyro_pid_receiver.get().await;
        let _setpoint_message = ctx.setpoint_receiver.get().await;
        // TODO: subscribe to global_context messages as well.

        // Update the OSD with the latest data.
        ctx.osd.update_display();
        //println!("OSD [50Hz]: Latest Gyro X: {}", data.gyro_rps.x);
        //info!("   OSD:      loop {loop_count}");
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
