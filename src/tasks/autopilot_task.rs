#![allow(unused)]

use log::info;
use radio_controllers::RadioControlMessage;
use vqm::Vector3df32;

use crate::{
    autopilot::pilot::Autopilot,
    sensor_data::{SensorDataItem, SensorDataSubscriber},
    tasks::dispatch::{GyroPidReceiver, SetpointReceiver},
    tasks::radio_task::AutopilotSender,
};

pub(crate) static AUTOPILOT_CTX: static_cell::StaticCell<AutopilotContext> = static_cell::StaticCell::new();

/// Context for Autopilot task.
pub struct AutopilotContext<'a> {
    pub sensor_data_subscriber: SensorDataSubscriber<'a>,
    pub gyro_pid_receiver: GyroPidReceiver,
    #[allow(unused)]
    pub setpoint_receiver: SetpointReceiver,
    pub autopilot_sender: AutopilotSender,
    pub autopilot: Autopilot,
}

/// Autopilot Placeholder.
#[embassy_executor::task]
pub async fn autopilot_task(ctx: &'static mut AutopilotContext<'static>) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(1));
    let delta_t = 0.01;
    let mut loop_count: u32 = 0;

    info!("AUTOPILOT:task started");
    loop {
        ticker.next().await;

        // Note: `try_next_message` is the non-blocking polling form.
        // If there is a message, it is removed and processed immediately.
        // Lagged (missed) messages are ignored.
        #[cfg(any(feature = "barometer", feature = "gps", feature = "rangefinder"))]
        if let Some(wait_result) = ctx.sensor_data_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(event) = wait_result
        {
            // TODO: choose position or altitude kalman filter based on settings
            match event {
                SensorDataItem::Barometer(barometer_data) => {
                    ctx.autopilot.altitude_kalman_filter.correct_altitude_using_barometer(barometer_data.altitude_m);
                    ctx.autopilot.position_kalman_filter.correct_altitude_using_barometer(barometer_data.altitude_m);
                }
                SensorDataItem::Rangefinder(rangefinder_data) => {
                    let rangefinder_base_altitude_m = 0.0_f32;
                    let altitude = rangefinder_base_altitude_m + rangefinder_data.distance_m;
                    ctx.autopilot.altitude_kalman_filter.correct_altitude_using_rangefinder(altitude);
                    ctx.autopilot.position_kalman_filter.correct_altitude_using_rangefinder(altitude);
                }
                SensorDataItem::GpsPosition(gps_position) => {
                    ctx.autopilot.altitude_kalman_filter.correct_altitude_using_gps(gps_position.position.z);
                    ctx.autopilot.position_kalman_filter.correct_position_using_gps(gps_position.position);
                }
                _ => {
                    // Message type of interest to other subscribers, but not to me so intentionally do nothing,
                    // this consumes the message and removes it from the queue.
                }
            }
        }
        #[cfg(any(feature = "barometer", feature = "gps", feature = "rangefinder"))]
        {
            if let Some(gyro_pid_message) = ctx.gyro_pid_receiver.try_get() {
                let vertical_acceleration = gyro_pid_message.acc.z;

                let estimate = ctx.autopilot.altitude_kalman_filter.predict(vertical_acceleration, delta_t);
                let Vector3df32 { x: estimated_vertical_speed, y: estimated_altitude, z: _estimated_bias } = estimate;

                // TODO: choose type of autopilot control based on settings: altitude hold, position hold, or path following.
                let throttle_stick = ctx.autopilot.altitude_controller.update(
                    estimated_altitude,
                    estimated_vertical_speed,
                    gyro_pid_message.orientation,
                    delta_t,
                );

                // Send the radio control message. This will be picked by the radio task.
                let radio_control_message = RadioControlMessage { throttle_stick, ..Default::default() };
                ctx.autopilot_sender.send(radio_control_message);
            }
        }

        if loop_count.is_multiple_of(200) {
            info!("AUTOPILOT:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
