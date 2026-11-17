#![cfg(feature = "osd")]

use static_cell::StaticCell;
use vqm::UnitQuaternionf32;

use super::{
    gyro_pid::{GyroPidReceiver, SetpointReceiver, gyro_pid_receiver, setpoint_receiver},
    rx::{RxMessageReceiver, rx_message_receiver},
};
use crate::{
    config::GLOBAL_CONFIG,
    display::{Display, DisplayPortLayer, DisplayPortMutex},
    flight::{ArmingFlags, RxMessage},
    osd::{Osd, OsdDrawContext, OsdElements, OsdState},
    tasks::errors::TaskInitError,
};

#[cfg(feature = "optical_flow")]
use super::optical_flow::{OpticalFlowSubscriber, optical_flow_subscriber};

#[cfg(feature = "rangefinder")]
use super::rangefinder::{RangefinderSubscriber, rangefinder_subscriber};

#[cfg(feature = "barometer")]
use super::barometer::{BarometerSubscriber, barometer_subscriber};

#[cfg(feature = "battery")]
use crate::{
    battery_sensors::BatteryMessage,
    tasks::battery::{BatterySubscriber, battery_subscriber},
};

#[cfg(feature = "gps")]
use super::gps::{GpsSubscriber, gps_subscriber};

static OSD_CTX: StaticCell<OsdContext> = StaticCell::new();
/// Context for OSD task.
#[allow(unused)]
pub struct OsdContext {
    gyro_pid_receiver: GyroPidReceiver,
    setpoint_receiver: SetpointReceiver,
    rx_receiver: RxMessageReceiver,
    rx_message: RxMessage,
    #[cfg(feature = "barometer")]
    barometer_subscriber: BarometerSubscriber,
    #[cfg(feature = "battery")]
    battery_subscriber: BatterySubscriber,
    #[cfg(feature = "battery")]
    battery_message: BatteryMessage,
    #[cfg(feature = "gps")]
    gps_subscriber: GpsSubscriber,
    #[cfg(feature = "optical_flow")]
    optical_flow_subscriber: OpticalFlowSubscriber,
    #[cfg(feature = "rangefinder")]
    rangefinder_subscriber: RangefinderSubscriber,
    osd: Osd,
    osd_state: OsdState,
    /// Subsystem handling layout, tracking, and rendering of individual OSD items.
    osd_elements: OsdElements,
    display_port_mutex: &'static DisplayPortMutex,
}

pub async fn init(display_port_mutex: &'static DisplayPortMutex) -> Result<&'static mut OsdContext, TaskInitError> {
    let display_port = display_port_mutex.lock().await;
    let background_layer_supported = display_port.layer_supported(DisplayPortLayer::Background);

    #[rustfmt::skip]
    let ctx = OsdContext {
        gyro_pid_receiver: gyro_pid_receiver()?,
        setpoint_receiver: setpoint_receiver()?,
        rx_receiver: rx_message_receiver()?,
        rx_message : RxMessage::new(),
        #[cfg(feature = "barometer")] barometer_subscriber: barometer_subscriber()?,
        #[cfg(feature = "battery")] battery_subscriber: battery_subscriber()?,
        #[cfg(feature = "battery")] battery_message: BatteryMessage::new(),
        #[cfg(feature = "gps")] gps_subscriber: gps_subscriber()?,
        #[cfg(feature = "optical_flow")] optical_flow_subscriber: optical_flow_subscriber()?,
        #[cfg(feature = "rangefinder")] rangefinder_subscriber: rangefinder_subscriber()?,
        osd: Osd::new(),
        osd_state: OsdState::default(),
        osd_elements: OsdElements::new(background_layer_supported),
        display_port_mutex,
    };

    Ok(OSD_CTX.init(ctx))
}

/// OSD Task Placeholder.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut OsdContext) {
    const TASK_FREQUENCY_HZ: u64 = 50;

    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(TASK_FREQUENCY_HZ));
    let mut loop_count: u32 = 0;

    let mut orientation = UnitQuaternionf32::default();

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
            if let Some(embassy_sync::pubsub::WaitResult::Message(battery_message)) =
                ctx.battery_subscriber.try_next_message()
            {
                ctx.battery_message = battery_message;
            }

            // Get the latest messages without consuming the notifications.
            if let Some(gyro_pid_message) = ctx.gyro_pid_receiver.try_get() {
                orientation = gyro_pid_message.orientation;
            }

            if let Some(rx) = ctx.rx_receiver.try_get() {
                ctx.rx_message = rx;
            }

            if ctx.osd_state.start_frame() {
                // Construct the draw context borrowing the display port.
                let draw_context = OsdDrawContext {
                    orientation,
                    arming_flags,
                    rx_message: ctx.rx_message,
                    #[cfg(feature = "battery")]
                    battery_message: ctx.battery_message,
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
                    let time_us = embassy_time::Instant::now().as_micros();
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
            log::info!("             OSD:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1);
    }
}
