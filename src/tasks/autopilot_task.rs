#![cfg(feature = "autopilot")]

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    watch::{Receiver, Sender, Watch},
};
use radio_controllers::RcModesArray;
use vqm::Vector3df32;

use crate::tasks::{
    flight_control_task::FlightControlReceiver,
    gyro_pid_task::{GyroPidReceiver, SetpointReceiver},
};

use crate::autopilot::pilot::Autopilot;

use crate::flight::FlightControlMessage;

#[cfg(feature = "barometer")]
use crate::tasks::barometer_task::BarometerSubscriber;

#[cfg(feature = "gps")]
use crate::{gps::GpsMessage, tasks::gps_task::GpsSubscriber};

#[cfg(feature = "optical_flow")]
use crate::tasks::optical_flow_task::OpticalFlowSubscriber;

#[cfg(feature = "rangefinder")]
use crate::tasks::rangefinder_task::RangefinderSubscriber;

const AUTOPILOT_WATCH_COUNT: usize = 1;
static AUTOPILOT_WATCH: Watch<CriticalSectionRawMutex, FlightControlMessage, AUTOPILOT_WATCH_COUNT> = Watch::new();

type AutopilotSender = Sender<'static, CriticalSectionRawMutex, FlightControlMessage, AUTOPILOT_WATCH_COUNT>;
pub fn autopilot_sender() -> AutopilotSender {
    AUTOPILOT_WATCH.sender()
}

pub type AutopilotReceiver = Receiver<'static, CriticalSectionRawMutex, FlightControlMessage, AUTOPILOT_WATCH_COUNT>;

#[allow(clippy::expect_used)]
pub fn autopilot_receiver() -> AutopilotReceiver {
    AUTOPILOT_WATCH.receiver().expect("autopilot_receiver failed")
}

/// Context for Autopilot task.
pub struct AutopilotContext<'a> {
    pub gyro_pid_receiver: GyroPidReceiver,
    #[allow(unused)]
    pub setpoint_receiver: SetpointReceiver,
    pub flight_control_receiver: FlightControlReceiver,
    pub autopilot_sender: AutopilotSender,
    pub autopilot: Autopilot,
    #[cfg(feature = "barometer")]
    pub barometer_subscriber: BarometerSubscriber<'a>,
    #[cfg(feature = "gps")]
    pub gps_subscriber: GpsSubscriber<'a>,
    #[allow(unused)]
    #[cfg(feature = "optical_flow")]
    pub optical_flow_subscriber: OpticalFlowSubscriber<'a>,
    #[cfg(feature = "rangefinder")]
    pub rangefinder_subscriber: RangefinderSubscriber<'a>,
}

/// Autopilot Placeholder.
#[embassy_executor::task]
pub async fn autopilot_task(ctx: &'static mut AutopilotContext<'static>) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(1));
    let delta_t = 0.001;
    let mut loop_count: u32 = 0;

    #[allow(unused)]
    let mut altitude_hold = false;
    #[cfg(feature = "gps")]
    let mut position_hold = false;

    log::info!("AUTOPILOT:task started");
    loop {
        ticker.next().await;

        // for the autopilot to provide any functionality, it has to have at least one of: barometer, gps, optical_flow, or rangefinder.
        #[cfg(any(feature = "barometer", feature = "gps", feature = "optical_flow", feature = "rangefinder"))]
        {
            if let Some(gyro_pid_message) = ctx.gyro_pid_receiver.try_get() {
                let vertical_acceleration = gyro_pid_message.acc.z;

                let Vector3df32 { x: estimated_vertical_speed, y: estimated_altitude, z: _estimated_bias } =
                    ctx.autopilot.altitude_kalman_filter.predict(vertical_acceleration, delta_t);

                // Check if the rc_modes have changed.
                if let Some(flight_control_message) = ctx.flight_control_receiver.try_changed() {
                    let rc_modes = flight_control_message.rc_modes;
                    altitude_hold = rc_modes.test(RcModesArray::ALTITUDE_HOLD);
                    #[cfg(feature = "gps")]
                    {
                        position_hold = rc_modes.test(RcModesArray::POSITION_HOLD)
                            | rc_modes.test(RcModesArray::GPS_RESCUE)
                            | rc_modes.test(RcModesArray::AUTOPILOT);
                    }
                }
                if altitude_hold {
                    let throttle_stick = ctx.autopilot.altitude_controller.update(
                        estimated_altitude,
                        estimated_vertical_speed,
                        gyro_pid_message.orientation,
                        delta_t,
                    );

                    // Send the flight control message. This will be picked by the radio task.
                    let flight_control_message = FlightControlMessage { throttle_stick, ..Default::default() };
                    ctx.autopilot_sender.send(flight_control_message);
                }
            }
        }

        // Note: `try_next_message` is the non-blocking polling form.
        // If there is a message, it is removed and processed immediately.
        // Lagged (missed) messages are ignored.
        #[cfg(feature = "barometer")]
        if let Some(wait_result) = ctx.barometer_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(barometer_message) = wait_result
        {
            ctx.autopilot.altitude_kalman_filter.correct_altitude_using_barometer(barometer_message.altitude_m);
            #[cfg(feature = "gps")]
            ctx.autopilot.position_kalman_filter.correct_altitude_using_barometer(barometer_message.altitude_m);
        }

        #[cfg(feature = "rangefinder")]
        if let Some(wait_result) = ctx.rangefinder_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(rangefinder_message) = wait_result
        {
            let rangefinder_base_altitude_m = 0.0_f32;
            let altitude = rangefinder_base_altitude_m + rangefinder_message.distance_m;
            ctx.autopilot.altitude_kalman_filter.correct_altitude_using_rangefinder(altitude);
            #[cfg(feature = "gps")]
            ctx.autopilot.position_kalman_filter.correct_altitude_using_rangefinder(altitude);
        }

        #[cfg(feature = "gps")]
        if let Some(wait_result) = ctx.gps_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(event) = wait_result
        {
            // TODO: choose position or altitude kalman filter based on settings
            if let GpsMessage::GpsPosition(gps_position) = event {
                if altitude_hold {
                    ctx.autopilot.altitude_kalman_filter.correct_altitude_using_gps(gps_position.position.z);
                } else if position_hold {
                    ctx.autopilot.position_kalman_filter.correct_position_using_gps(gps_position.position);
                }
            } else {
                // Message type of interest to other subscribers, but not to me so intentionally do nothing,
                // this consumes the message and removes it from the queue.
            }
        }

        if loop_count.is_multiple_of(200) {
            log::info!("     AUTOPILOT:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
