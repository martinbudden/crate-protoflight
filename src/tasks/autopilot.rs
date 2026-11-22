#![cfg(feature = "autopilot")]

#[allow(unused)]
use embassy_sync::pubsub::WaitResult;
use embassy_time::Instant;
use vqm::Vector3f32;

#[allow(unused)]
use embassy_futures::yield_now;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    signal::Signal,
    watch::{Receiver, Sender, Watch},
};
use static_cell::StaticCell;

use crate::{
    autopilot::Autopilot,
    flight::RxMessage,
    tasks::{
        errors::TaskInitError,
        failsafe::{FailsafeSubscriber, failsafe_subscriber},
        gyro_pid::{GyroPidReceiver, ImuIntegralReceiver, gyro_pid_receiver, imu_integral_receiver},
        rx::{RxMessageReceiver, rx_message_receiver},
    },
};

static AUTOPILOT_CTX: StaticCell<AutopilotContext> = StaticCell::new();

#[cfg(any(feature = "gps", feature = "optical_flow"))]
use sensor_fusion::KalmanFilterXyzWithSensorsf32;
#[cfg(not(any(feature = "gps", feature = "optical_flow")))]
use sensor_fusion::KalmanFilterZWithSensorsf32;

#[cfg(feature = "barometer")]
use crate::tasks::barometer::{BarometerSubscriber, barometer_subscriber};

#[cfg(feature = "gps")]
use crate::{
    gps::GpsMessage,
    tasks::gps::{GpsSubscriber, gps_subscriber},
};

#[cfg(feature = "optical_flow")]
use crate::tasks::optical_flow::{OpticalFlowSubscriber, optical_flow_subscriber};

#[cfg(feature = "rangefinder")]
use crate::tasks::rangefinder::{RangefinderSubscriber, rangefinder_subscriber};

pub static KALMAN_ACC_BIAS_SIGNAL: Signal<CriticalSectionRawMutex, KalmanAccBiasNedMessage> = Signal::new();

const AUTOPILOT_WATCH_COUNT: usize = 1;
static AUTOPILOT_WATCH: Watch<CriticalSectionRawMutex, RxMessage, AUTOPILOT_WATCH_COUNT> = Watch::new();

type AutopilotSender = Sender<'static, CriticalSectionRawMutex, RxMessage, AUTOPILOT_WATCH_COUNT>;
pub fn autopilot_sender() -> AutopilotSender {
    AUTOPILOT_WATCH.sender()
}

pub type AutopilotReceiver = Receiver<'static, CriticalSectionRawMutex, RxMessage, AUTOPILOT_WATCH_COUNT>;

pub fn autopilot_receiver() -> Result<AutopilotReceiver, TaskInitError> {
    AUTOPILOT_WATCH.receiver().ok_or(TaskInitError::AutopilotWatchCountTooLow)
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
pub struct KalmanAccBiasNedMessage {
    pub acc_bias_ned_mps2: Vector3f32,
}

impl Default for KalmanAccBiasNedMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl KalmanAccBiasNedMessage {
    pub const fn new() -> Self {
        Self { acc_bias_ned_mps2: Vector3f32 { x: 0.0, y: 0.0, z: 0.0 } }
    }
}

/// Context for Autopilot task.
#[rustfmt::skip]
pub struct AutopilotContext {
    gyro_pid_receiver: GyroPidReceiver,
    imu_integral_receiver: ImuIntegralReceiver,
    rx_receiver: RxMessageReceiver,
    failsafe_subscriber: FailsafeSubscriber,
    autopilot_sender: AutopilotSender,
    autopilot: Autopilot,
    #[cfg(feature = "barometer")] pub barometer_subscriber: BarometerSubscriber,
    #[cfg(feature = "gps")] pub gps_subscriber: GpsSubscriber,
    #[cfg(feature = "optical_flow")] pub optical_flow_subscriber: OpticalFlowSubscriber,
    #[cfg(feature = "rangefinder")] pub rangefinder_subscriber: RangefinderSubscriber,
    // If we have GPS or optical flow, then use a `KalmanFilterXyz`.
    #[cfg(any(feature = "gps", feature = "optical_flow"))]
    pub kalman_filter: KalmanFilterXyzWithSensorsf32,
    // If we don't have either gps or optical flow, then we can't correct position in the xy plane,
    // so there is no point in having the overhead of a full 3D Kalman filter, so we just use a `KalmanFilterZ`.
    #[cfg(not(any(feature = "gps", feature = "optical_flow")))]
    pub kalman_filter: KalmanFilterZWithSensorsf32,
}

pub fn init() -> Result<&'static mut AutopilotContext, TaskInitError> {
    #[rustfmt::skip]
    let ctx = AutopilotContext {
        gyro_pid_receiver: gyro_pid_receiver()?,
        imu_integral_receiver: imu_integral_receiver()?,
        rx_receiver: rx_message_receiver()?,
        failsafe_subscriber: failsafe_subscriber()?,
        autopilot_sender: autopilot_sender(),
        autopilot: Autopilot::new(),
        #[cfg(feature = "barometer")] barometer_subscriber: barometer_subscriber()?,
        #[cfg(feature = "gps")] gps_subscriber: gps_subscriber()?,
        #[cfg(feature = "optical_flow")] optical_flow_subscriber: optical_flow_subscriber()?,
        #[cfg(feature = "rangefinder")] rangefinder_subscriber: rangefinder_subscriber()?,
        #[cfg(any(feature = "gps", feature = "optical_flow"))]
        kalman_filter: KalmanFilterXyzWithSensorsf32::new(),
        #[cfg(not(any(feature = "gps", feature = "optical_flow")))]
        kalman_filter: KalmanFilterZWithSensorsf32::new(),
    };
    Ok(AUTOPILOT_CTX.init(ctx))
}

// TODO: make this run on receipt of the integrated_imu message, rather than on a timer.
/// Autopilot Placeholder.
#[allow(clippy::too_many_lines)]
#[allow(unused)]
#[embassy_executor::task]
pub async fn run(ctx: &'static mut AutopilotContext) {
    let delta_t = 0.002;
    #[allow(clippy::cast_possible_truncation,clippy::cast_sign_loss)]
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis((1000.0*delta_t) as u64));
    let mut loop_count: u32 = 0;

    log::info!("   AUTOPILOT: task started");
    loop {
        ticker.next().await;
        let time_us = Instant::now().as_micros();

        // *********************
        // Kalman Filter Predict
        // *********************

        if let Some(imu_integral_message) = ctx.imu_integral_receiver.try_get() {
            let imu_integral = imu_integral_message.imu_integral;
            ctx.kalman_filter.predict(time_us, &imu_integral, delta_t);
            // signal the acc bias so the gyro_pid task use it to update the IMU integrator.
            let acc_bias_ned_mps2 = ctx.kalman_filter.state().acc_bias_ned_mps2();
            KALMAN_ACC_BIAS_SIGNAL.signal(KalmanAccBiasNedMessage { acc_bias_ned_mps2 });
            yield_now().await;
        }

        // *********************
        // Kalman Filter Correct
        // *********************

        // Note: `try_next_message` is the non-blocking polling form.
        // If there is a message, it is removed and processed immediately.
        // Lagged (missed) messages are ignored.
        #[cfg(feature = "barometer")]
        if let Some(wait_result) = ctx.barometer_subscriber.try_next_message()
            && let WaitResult::Message(barometer_message) = wait_result
        {
            _ = ctx.kalman_filter.correct_altitude_using_barometer(
                barometer_message.timestamp.as_micros(),
                barometer_message.altitude_m,
            );
            yield_now().await;
        }

        #[cfg(feature = "rangefinder")]
        if let Some(wait_result) = ctx.rangefinder_subscriber.try_next_message()
            && let WaitResult::Message(rangefinder_message) = wait_result
        {
            // TODO: set rangefinder_base_altitude_m
            let rangefinder_base_altitude_m = 0.0_f32;
            let altitude = rangefinder_base_altitude_m + rangefinder_message.distance_m;
            _ = ctx
                .kalman_filter
                .correct_altitude_using_rangefinder(rangefinder_message.timestamp.as_micros(), altitude);
            yield_now().await;
        }

        #[cfg(feature = "gps")]
        if let Some(wait_result) = ctx.gps_subscriber.try_next_message()
            && let WaitResult::Message(gps_message) = wait_result
        {
            // TODO: choose position or altitude kalman filter based on settings
            if let GpsMessage::PositionVelocity(timestamp_us, gps_position, gps_velocity) = gps_message {
                _ = ctx.kalman_filter.correct_position_and_velocity_using_gps(timestamp_us, gps_position, gps_velocity);
                yield_now().await;
            } else {
                // Message type of interest to other subscribers, but not to me so intentionally do nothing,
                // this consumes the message and removes it from the queue.
            }
        }

        #[cfg(feature = "optical_flow")]
        if let Some(wait_result) = ctx.optical_flow_subscriber.try_next_message()
            && let WaitResult::Message(optical_flow_message) = wait_result
        {
            _ = ctx.kalman_filter.correct_velocity_using_optical_flow(
                optical_flow_message.timestamp.as_micros(),
                optical_flow_message.velocity,
            );
            yield_now().await;
        }

        // ****************
        // Autopilot Update
        // ****************

        // for the autopilot to provide any functionality, it has to have at least one of: barometer, gps, optical_flow, or rangefinder.
        #[cfg(any(feature = "barometer", feature = "gps", feature = "optical_flow", feature = "rangefinder"))]
        {
            if let Some(gyro_pid_message) = ctx.gyro_pid_receiver.try_get() {
                let rx_message = ctx.rx_receiver.try_changed();
                let mut failsafe_state = crate::tasks::FailsafeState::Idle;
                if let Some(WaitResult::Message(failsafe_message)) = ctx.failsafe_subscriber.try_next_message() {
                    failsafe_state = failsafe_message.state;
                }
                if let Some(rc_controls) = ctx.autopilot.update(
                    rx_message,
                    failsafe_state,
                    &ctx.kalman_filter.state(),
                    gyro_pid_message.orientation,
                    delta_t,
                ) {
                    // Send the flight control message. This will be picked by the radio task.
                    let rx_message = RxMessage { rc_controls, ..Default::default() };
                    ctx.autopilot_sender.send(rx_message);
                }
            }
        }

        if loop_count.is_multiple_of(200) {
            log::info!("       AUTOPILOT:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1);
    }
}
