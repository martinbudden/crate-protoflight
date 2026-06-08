#![cfg(feature = "osd")]

use log::info;
use vqm::Quaternionf32;

use crate::flight::ArmingFlags;
use crate::osd::{Osd, OsdDrawContext};
use crate::tasks::gyro_pid_task::{GyroPidReceiver, SetpointReceiver};

#[cfg(feature = "barometer")]
use crate::tasks::barometer_task::BarometerDataSubscriber;

#[cfg(feature = "gps")]
use crate::tasks::gps_task::GpsDataSubscriber;
use crate::tasks::init::SharedDisplay;

/// Context for OSD task.
#[allow(unused)]
pub struct OsdContext<'a> {
    pub gyro_pid_receiver: GyroPidReceiver,
    pub setpoint_receiver: SetpointReceiver,
    #[cfg(feature = "barometer")]
    pub barometer_data_subscriber: BarometerDataSubscriber<'a>,
    #[cfg(feature = "gps")]
    pub gps_data_subscriber: GpsDataSubscriber<'a>,
    pub osd: Osd,
}

/// OSD Task Placeholder.
///

/*#[embassy_executor::task]
pub async fn osd_task(ctx: &'static mut OsdContext<'static>) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut loop_count: u32 = 0;

    //println!("OSD: Started at 50Hz.");
    info!("      OSD: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;

        // Get the latest messages without consuming the notifications.
        let orientation = if let Some(gyro_pid_message) = ctx.gyro_pid_receiver.try_get() {
            gyro_pid_message.orientation
        } else {
            Quaternionf32::default()
        };

        #[cfg(feature = "max7456")]
        let mut display_port = DisplayPortMax7456::new();
        #[cfg(not(feature = "max7456"))]
        let mut display_port = DisplayPortMock::new();

        let arming_flags = ArmingFlags::new();
        let mut draw_context = OsdDrawContext { display_port: &mut display_port, orientation, arming_flags };
        // Update the OSD with the latest data.
        let time_microseconds = 0_u32;
        ctx.osd.update_display(&mut draw_context, time_microseconds);

        if loop_count.is_multiple_of(10) {
            info!("      OSD:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
*/

#[embassy_executor::task]
pub async fn osd_task(
    ctx: &'static mut OsdContext<'static>,
    display_mutex: &'static SharedDisplay, // Perfectly legal now!
) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut loop_count: u32 = 0;

    info!("      OSD: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;

        // 1. Check your configuration or user setting
        let osd_enabled = true; //check_if_osd_active();

        if osd_enabled {
            // 2. Lock the display dynamically.
            // While this guard lives, cms_task cannot touch the display port.
            let mut display_guard = display_mutex.lock().await;

            // Get the latest messages without consuming the notifications.
            let orientation = if let Some(gyro_pid_message) = ctx.gyro_pid_receiver.try_get() {
                gyro_pid_message.orientation
            } else {
                Quaternionf32::default()
            };
            let arming_flags = ArmingFlags::new();

            // 3. Construct your context directly borrowing the inner guard
            let mut draw_context = OsdDrawContext {
                display_port: &mut *display_guard, // Deref to get &mut Driver
                orientation,
                arming_flags,
            };

            #[allow(clippy::cast_possible_truncation)]
            let time_microseconds = embassy_time::Instant::now().as_micros() as u32;
            ctx.osd.update_display(&mut draw_context, time_microseconds).await;

            if loop_count.is_multiple_of(10) {
                info!("      OSD:      loop {loop_count}");
            }
            loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
            // display_guard is automatically dropped at the end of this block,
            // releasing the Mutex lock for cms_task!
        }
    }
}
