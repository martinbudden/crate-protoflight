#![cfg(feature = "osd")]

use vqm::Quaternionf32;

use crate::flight::ArmingFlags;
use crate::osd::{Osd, OsdDrawContext};

use crate::tasks::{
    gyro_pid_task::{GyroPidReceiver, SetpointReceiver},
    init::DisplayPortMutex,
};

#[cfg(feature = "optical_flow")]
use crate::tasks::optical_flow_task::OpticalFlowSubscriber;

#[cfg(feature = "rangefinder")]
use crate::tasks::rangefinder_task::RangefinderSubscriber;

#[cfg(feature = "barometer")]
use crate::tasks::barometer_task::BarometerSubscriber;

#[cfg(feature = "battery")]
use crate::{sensors::BatteryMessage, tasks::battery_task::BatterySubscriber};

#[cfg(feature = "gps")]
use crate::tasks::gps_task::GpsSubscriber;

/// Context for OSD task.
#[allow(unused)]
pub struct OsdContext<'a> {
    pub gyro_pid_receiver: GyroPidReceiver,
    pub setpoint_receiver: SetpointReceiver,
    #[cfg(feature = "barometer")]
    pub barometer_subscriber: BarometerSubscriber<'a>,
    #[cfg(feature = "battery")]
    pub battery_subscriber: BatterySubscriber<'a>,
    #[cfg(feature = "gps")]
    pub gps_subscriber: GpsSubscriber<'a>,
    #[cfg(feature = "optical_flow")]
    pub optical_flow_subscriber: OpticalFlowSubscriber<'a>,
    #[cfg(feature = "rangefinder")]
    pub rangefinder_subscriber: RangefinderSubscriber<'a>,
    pub osd: Osd,
}

/// OSD Task Placeholder.
///

/*#[embassy_executor::task]
pub async fn osd_task(ctx: &'static mut OsdContext<'static>) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut loop_count: u32 = 0;

    //println!("OSD: Started at 50Hz.");
    log::info!("      OSD: task started");
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
            log::info!("      OSD:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
*/

#[embassy_executor::task]
pub async fn osd_task(ctx: &'static mut OsdContext<'static>, display_port_mutex: &'static DisplayPortMutex) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut loop_count: u32 = 0;

    let mut battery_message = BatteryMessage::new();
    let mut orientation = Quaternionf32::default();

    log::info!("      OSD: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;

        // check_if_osd_active();
        let osd_enabled = true;

        if osd_enabled {
            // Lock the display port, while this guard lives other tasks cannot use the display port.
            let mut display_port_guard = display_port_mutex.lock().await;

            // TODO: replace these placeholder values with real values
            let arming_flags = ArmingFlags::new();

            // Get the latest messages without consuming the notifications.
            if let Some(gyro_pid_message) = ctx.gyro_pid_receiver.try_get() {
                orientation = gyro_pid_message.orientation;
            }

            #[cfg(feature = "battery")]
            if let Some(wait_result) = ctx.battery_subscriber.try_next_message()
                && let embassy_sync::pubsub::WaitResult::Message(battery_data) = wait_result
            {
                battery_message = battery_data;
            }

            // Construct the draw context borrowing the display port.
            let mut draw_context = OsdDrawContext {
                display_port: &mut *display_port_guard,
                orientation,
                arming_flags,
                #[cfg(feature = "battery")]
                battery_message,
            };

            #[allow(clippy::cast_possible_truncation)]
            let time_microseconds = embassy_time::Instant::now().as_micros() as u32;
            ctx.osd.update_display(&mut draw_context, time_microseconds).await;

            // display_port_guard is automatically dropped at the end of this block,
            // releasing the Mutex lock for other tasks.
        }

        if loop_count.is_multiple_of(10) {
            log::info!("           OSD:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
