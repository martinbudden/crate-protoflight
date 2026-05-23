#![allow(unused)]
/// Autopilot Placeholder.
///
use embassy_time::Duration;

use log::info;
use radio_controllers::RadioControlMessage;
use static_cell::StaticCell;
use vqm::Vector3df32;

use crate::{
    autopilot::pilot::Autopilot,
    dispatch::{GyroPidReceiver, SetpointReceiver},
    tasks::radio_task::AutopilotSender,
};

pub(crate) static AUTOPILOT_CTX: StaticCell<AutopilotContext> = StaticCell::new();

/// Context for Autopilot task.
pub struct AutopilotContext {
    pub gyro_pid_receiver: GyroPidReceiver,
    pub setpoint_receiver: SetpointReceiver,
    pub autopilot_sender: AutopilotSender,
    pub autopilot: Autopilot,
}

#[embassy_executor::task]
pub async fn autopilot_task(ctx: &'static mut AutopilotContext) {
    let mut ticker = embassy_time::Ticker::every(Duration::from_millis(1));
    let delta_t = 0.01;
    let mut loop_count: u32 = 0;

    info!("AUTOPILOT:task started");
    loop {
        ticker.next().await;

        //let setpoint_message = ctx.setpoint_receiver.get().await;
        // Peek(get) the latest messages without consuming the notifications.
        #[cfg(any(feature = "barometer", feature = "gps"))]
        let gyro_pid_message = ctx.gyro_pid_receiver.get().await;

        #[cfg(feature = "barometer")]
        {
            let barometer_altitude = 0.0;
            let vertical_acceleration = gyro_pid_message.acc.z;
            let estimate =
                ctx.autopilot.altitude_kalman_filter.update(barometer_altitude, vertical_acceleration, delta_t);
            let Vector3df32 { x: estimated_vertical_speed, y: estimated_altitude, z: _estimated_bias } = estimate;
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
        //#[cfg(feature = "gps")]

        if loop_count.is_multiple_of(200) {
            info!("AUTOPILOT:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
