#![cfg(feature = "osd")]

#[cfg(feature = "battery")]
use embassy_sync::pubsub::WaitResult;
use static_cell::StaticCell;
use vqm::Quaternionf32;

use crate::{
    config::GLOBAL_CONFIG,
    display::{Display, DisplayPortLayer, DisplayPortMutex},
    flight::{ArmingFlags, RxMessage},
    osd::{Osd, OsdDrawContext, OsdElements, OsdState},
    tasks::{
        gyro_pid::{GyroPidReceiver, SetpointReceiver, gyro_pid_receiver, setpoint_receiver},
        rx::{RxMessageReceiver, rx_message_receiver},
    },
};

#[cfg(feature = "optical_flow")]
use crate::tasks::optical_flow::{OpticalFlowSubscriber, optical_flow_subscriber};

#[cfg(feature = "rangefinder")]
use crate::tasks::rangefinder::{RangefinderSubscriber, rangefinder_subscriber};

#[cfg(feature = "barometer")]
use crate::tasks::barometer::{BarometerSubscriber, barometer_subscriber};

#[cfg(feature = "battery")]
use crate::{
    sensors::BatteryMessage,
    tasks::battery::{BatterySubscriber, battery_subscriber},
};

#[cfg(feature = "gps")]
use crate::tasks::gps::{GpsSubscriber, gps_subscriber};

static OSD_CTX: StaticCell<OsdContext> = StaticCell::new();
/// Context for OSD task.
#[allow(unused)]
pub struct OsdContext {
    pub gyro_pid_receiver: GyroPidReceiver,
    pub setpoint_receiver: SetpointReceiver,
    pub rx_receiver: RxMessageReceiver,
    #[cfg(feature = "barometer")]
    pub barometer_subscriber: BarometerSubscriber,
    #[cfg(feature = "battery")]
    pub battery_subscriber: BatterySubscriber,
    #[cfg(feature = "gps")]
    pub gps_subscriber: GpsSubscriber,
    #[cfg(feature = "optical_flow")]
    pub optical_flow_subscriber: OpticalFlowSubscriber,
    #[cfg(feature = "rangefinder")]
    pub rangefinder_subscriber: RangefinderSubscriber,
    pub osd: Osd,
    pub osd_state: OsdState,
    /// Subsystem handling layout, tracking, and rendering of individual OSD items.
    pub osd_elements: OsdElements,
    pub display_port_mutex: &'static DisplayPortMutex,
}

impl OsdContext {
    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    pub fn new(display_port_mutex: &'static DisplayPortMutex,
        background_layer_supported: bool,
    ) -> Self {
        Self {
            gyro_pid_receiver:gyro_pid_receiver(),
            setpoint_receiver:setpoint_receiver(),
            rx_receiver:rx_message_receiver(),
            #[cfg(feature = "barometer")] barometer_subscriber:barometer_subscriber(),
            #[cfg(feature = "battery")] battery_subscriber:battery_subscriber(),
            #[cfg(feature = "gps")] gps_subscriber:gps_subscriber(),
            #[cfg(feature = "optical_flow")] optical_flow_subscriber:optical_flow_subscriber(),
            #[cfg(feature = "rangefinder")] rangefinder_subscriber:rangefinder_subscriber(),
            osd: Osd::new(),
            osd_state: OsdState::default(),
            osd_elements: OsdElements::new(background_layer_supported),
            display_port_mutex,
        }
    }
}

pub async fn init(display_port_mutex: &'static DisplayPortMutex) -> &'static mut OsdContext {
    let display_port = display_port_mutex.lock().await;
    let background_layer_supported = display_port.layer_supported(DisplayPortLayer::Background);

    OSD_CTX.init(OsdContext::new(display_port_mutex, background_layer_supported))
}

/// OSD Task Placeholder.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut OsdContext) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut loop_count: u32 = 0;

    #[cfg(feature = "battery")]
    let mut battery_message = BatteryMessage::new();
    let mut orientation = Quaternionf32::default();
    let mut rx_message = RxMessage::new();

    log::info!("         OSD: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;

        // TODO: check_if_osd_active();
        let osd_enabled = true;

        if osd_enabled {
            // TODO: replace these placeholder values with real values
            let arming_flags = ArmingFlags::new();

            #[cfg(feature = "battery")]
            if let Some(WaitResult::Message(battery_data)) = ctx.battery_subscriber.try_next_message() {
                battery_message = battery_data;
            }

            // Get the latest messages without consuming the notifications.
            if let Some(gyro_pid_message) = ctx.gyro_pid_receiver.try_get() {
                orientation = gyro_pid_message.orientation;
            }

            if let Some(rx) = ctx.rx_receiver.try_get() {
                rx_message = rx;
            }

            if ctx.osd_state.start_frame() {
                // Construct the draw context borrowing the display port.
                let draw_context = OsdDrawContext {
                    orientation,
                    arming_flags,
                    rx_message,
                    #[cfg(feature = "battery")]
                    battery_message,
                };

                let osd_config = {
                    // lock().await takes approximately 10 to 50 CPU cycles if the lock is uncontested.
                    let global_config = GLOBAL_CONFIG.lock().await;
                    global_config.osd
                };

                // TODO: Investigate whether a complete OSD refresh can consistently complete
                // within one 50 Hz period. If not, consider spreading element updates
                // across multiple task iterations to reduce latency spikes.
                while ctx.osd_state != OsdState::Idle {
                    #[allow(clippy::cast_possible_truncation)]
                    let time_us = embassy_time::Instant::now().as_micros() as u32;
                    ctx.osd_state
                        .update_display_iteration(
                            &mut ctx.osd_elements,
                            &draw_context,
                            ctx.display_port_mutex,
                            &osd_config,
                            time_us,
                        )
                        .await;
                }
            }
        }
        if loop_count.is_multiple_of(50) {
            log::info!("           OSD:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}

/*#[embassy_executor::task]
pub async fn run(ctx: &'static mut OsdContext) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut loop_count: u32 = 0;

    //println!("OSD: Started at 50Hz.");
    log::info!("         OSD: task started");
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
