use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::WaitResult,
    watch::{Receiver, Sender, Watch},
};
use embassy_time::Instant;

use pidsk_controller::PdGainsf32;
use static_cell::StaticCell;

use imu_sensors::{AccFullScale, AccUnits, GyroFullScale, GyroUnits, ImuDevice, ImuError};
use motor_mixers::MotorMixerMessage;
use sensor_fusion::{MadgwickFilterf32, SensorFusion};
use simple_bitset::BitSet64;

#[cfg(feature = "rpm_filters")]
use motor_mixers::RpmNotchFilterBankConfig;

use crate::{
    boards::targets::BoardImu,
    config::{FastConfigItem, FastConfigSubscriber, fast_config_subscriber},
    flight::{FilterAccGyro, FlightController, ImuFilterBank, ImuFilterBankConfig, RcControls, VehicleControl},
    tasks::{
        GyroPidMessage, SetpointMessage,
        motor_mixer::MOTOR_MIXER_SIGNAL,
        rx::{RxMessageReceiver, rx_message_receiver},
    },
};

#[cfg(feature = "gps")]
use crate::tasks::gps::GPS_YAW_HEADING_SIGNAL;

// The gyro_pid watch has three clients: the blackbox, the autopilot, and the OSD.
const GYRO_PID_WATCH_COUNT: usize = 3;
// Watch<Mutex, DataType, MaxReceivers>
static GYRO_PID_WATCH: Watch<CriticalSectionRawMutex, GyroPidMessage, GYRO_PID_WATCH_COUNT> = Watch::new();

// Type aliases make the function signatures much easier to read.
type GyroPidSender = Sender<'static, CriticalSectionRawMutex, GyroPidMessage, GYRO_PID_WATCH_COUNT>;
pub fn gyro_pid_sender() -> GyroPidSender {
    GYRO_PID_WATCH.sender()
}

#[allow(unused)]
pub type GyroPidReceiver = Receiver<'static, CriticalSectionRawMutex, GyroPidMessage, GYRO_PID_WATCH_COUNT>;

#[allow(unused)]
#[allow(clippy::expect_used)]
pub fn gyro_pid_receiver() -> GyroPidReceiver {
    GYRO_PID_WATCH.receiver().expect("gyro_pid receiver failed")
}

const SETPOINT_WATCH_COUNT: usize = 3;
static SETPOINT_WATCH: Watch<CriticalSectionRawMutex, SetpointMessage, SETPOINT_WATCH_COUNT> = Watch::new();

type SetpointSender = Sender<'static, CriticalSectionRawMutex, SetpointMessage, SETPOINT_WATCH_COUNT>;
pub fn setpoint_sender() -> SetpointSender {
    SETPOINT_WATCH.sender()
}

pub type SetpointReceiver = Receiver<'static, CriticalSectionRawMutex, SetpointMessage, SETPOINT_WATCH_COUNT>;

#[allow(unused)]
#[allow(clippy::expect_used)]
pub fn setpoint_receiver() -> SetpointReceiver {
    SETPOINT_WATCH.receiver().expect("setpoint receiver failed")
}

static GYRO_PID_CTX: StaticCell<GyroPidContext<BoardImu>> = StaticCell::new();

/// Context for `gyro_pid` task.
#[allow(unused)]
pub struct GyroPidContext<I: ImuDevice> {
    pub imu: I,
    pub rx_receiver: RxMessageReceiver,
    pub gyro_pid_sender: GyroPidSender,
    pub setpoint_sender: SetpointSender,
    pub fast_config_subscriber: FastConfigSubscriber,
    pub imu_filters: ImuFilterBank,
    pub sensor_fusion: MadgwickFilterf32,
    pub flight_controller: FlightController,
    pub rc_controls: RcControls,
    pub rc_modes: BitSet64,
    pub gyro_pid_send_count: u32,
    pub gyro_pid_denominator: u32,
}

pub fn init(
    imu: BoardImu,
    imu_filter_bank_config: ImuFilterBankConfig,
    #[cfg(feature = "rpm_filters")] rpm_notch_filter_bank_config: RpmNotchFilterBankConfig,
    #[cfg(feature = "rpm_filters")] looptime_seconds: f32,
) -> &'static mut GyroPidContext<BoardImu> {
    let ctx = GyroPidContext {
        imu,
        rx_receiver: rx_message_receiver(),
        gyro_pid_sender: gyro_pid_sender(),
        setpoint_sender: setpoint_sender(),
        fast_config_subscriber: fast_config_subscriber(),

        #[cfg(feature = "rpm_filters")]
        imu_filters: ImuFilterBank::with_config_and_notch(
            imu_filter_bank_config,
            rpm_notch_filter_bank_config,
            looptime_seconds,
        ),
        #[cfg(not(feature = "rpm_filters"))]
        imu_filters: ImuFilterBank::with_config(imu_filter_bank_config),

        sensor_fusion: MadgwickFilterf32::new(),
        flight_controller: FlightController::new(),
        rc_controls: RcControls::new(),
        rc_modes: BitSet64::new(),
        gyro_pid_send_count: 0,
        gyro_pid_denominator: 10,
    };

    GYRO_PID_CTX.init(ctx)
}

/// The GYRO/PID task.
/// The `gyro_pid` task calculates the motor commands, sends them immediately to the `motor_mixer` task
/// and then updates the `GyroPidMessage` and sends it, so it can be picked up the the Blackbox and the OSD.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut GyroPidContext<BoardImu>) {
    log::info!("    GYRO_PID: task started");

    let sample_rates = ctx.imu.init(8000, GyroFullScale::Max, GyroUnits::Rps, AccFullScale::Max, AccUnits::G).await;
    let (gyro_rate_hz, _acc_rate_hz) = match sample_rates {
        Ok((gyro_rate_hz, acc_rate_hz)) => (gyro_rate_hz, acc_rate_hz),
        Err(_err) => (1000, 1000),
    };

    #[allow(clippy::cast_precision_loss)]
    let delta_t: f32 = 1.0 / (gyro_rate_hz as f32);

    // TODO: this ticker wait should be replaced by an interrupt driven DMA read from the IMU.
    // I'm using a ticker like this during development to keep things simple.
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(u64::from(gyro_rate_hz)));

    // This is the famous GYRO/PID loop!
    let mut loop_count: u32 = 0;
    loop {
        ticker.next().await;
        if let Err(err) = gyro_pid_loop_iteration(ctx, delta_t).await {
            // We really shouldn't get here, so consider so sort of failsafe handling.
            log::error!("GYRO_PID iteration failed: {err:?}");
        }
        if loop_count.is_multiple_of(1000) {
            log::info!("        GYRO_PID: loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1);
    }
}

async fn gyro_pid_loop_iteration(ctx: &mut GyroPidContext<BoardImu>, delta_t: f32) -> Result<(), ImuError> {
    // ****
    // The GYRO part of the GYRO/PID loop
    // ****

    let (acc, gyro_rps) = ctx.imu.read_acc_gyro().await?;

    // Save the unfiltered gyro value for telemetry.
    let gyro_rps_unfiltered = gyro_rps;

    // Filter the acc and gyro values. This includes RPM notch filtering, if that is enabled.
    let (acc, gyro_rps) = ctx.imu_filters.update(acc, gyro_rps, delta_t);

    // Check if there has been a yaw heading correction from the GPS, if so, apply it.
    #[cfg(feature = "gps")]
    if let Some(gps_yaw_heading) = GPS_YAW_HEADING_SIGNAL.try_take() {
        _ = ctx.sensor_fusion.correct_yaw(gps_yaw_heading.yaw_heading_radians, gps_yaw_heading.delta_t);
    }

    // Calculate the orientation quaternion using sensor fusion.
    let orientation = ctx.sensor_fusion.fuse_acc_gyro(acc, gyro_rps, delta_t);

    // ****
    // The PID part of the GYRO/PID loop
    // ****

    // If there are new control values from the radio then use them, otherwise keep using the old control values.
    if let Some(rx_message) = ctx.rx_receiver.try_changed() {
        ctx.rc_controls = rx_message.rc_controls;
        ctx.rc_modes = rx_message.rc_modes;
    }

    // Calculate the motor commands:
    // the flight controller updates its setpoints from the radio control_message
    // and then updates the PIDs using `gyro_rps` and `orientation`.
    // `setpoints_updated` is set if the setpoints have been updated because of a new radio_control_message,
    // or if the flight controller has updated the setpoints because of crash or spin recovery.
    let (motor_commands, setpoints_updated) =
        ctx.flight_controller.calculate_motor_commands(gyro_rps, orientation, delta_t, ctx.rc_controls, ctx.rc_modes);

    // Convert the motor commands calculated by the flight controller into a motor mixer message and send that message.
    // The signal will be picked up by the motor mixer task.
    // We signal every time round the GYRO/PID loop since the motor mixer also updates the RPM notch filters on this signal.
    MOTOR_MIXER_SIGNAL.signal(MotorMixerMessage::from(motor_commands));

    // Send the GyroPidMessage on a denominator (e.g., 1/8 = 1kHz)
    // This will be picked up by the Blackbox and the OSD.
    ctx.gyro_pid_send_count += 1;

    #[cfg(any(feature = "blackbox", feature = "osd"))]
    if ctx.gyro_pid_send_count >= ctx.gyro_pid_denominator {
        ctx.gyro_pid_send_count = 0;

        let roll_errors = ctx.flight_controller.roll.rate_pid.error();
        let pitch_errors = ctx.flight_controller.pitch.rate_pid.error();
        let yaw_errors = ctx.flight_controller.yaw_rate_pid.error();
        let pid_errors_p = [roll_errors.p, pitch_errors.p, yaw_errors.p];
        let pid_errors_i = [roll_errors.i, pitch_errors.i, yaw_errors.i];
        let pid_errors_d = [roll_errors.d, pitch_errors.d];

        let time_us = Instant::now().as_micros();
        let gyro_pid_message = GyroPidMessage {
            orientation,
            motor_commands,
            acc,
            gyro_rps,
            gyro_rps_unfiltered,
            pid_errors_p,
            pid_errors_i,
            pid_errors_d,
            time_us,
        };
        ctx.gyro_pid_sender.send(gyro_pid_message);

        if setpoints_updated {
            // Only send a setpoint_message when the setpoints have actually been updated.
            // This is picked up by the Blackbox.
            // TODO fill out missing SetpointMessage fields.
            let pid_errors_s = [roll_errors.s, pitch_errors.s, yaw_errors.s];
            let pid_errors_k = [roll_errors.k, pitch_errors.k, yaw_errors.k];
            let setpoint_message = SetpointMessage {
                time_us,
                rc_modes: ctx.rc_modes,
                setpoints: [
                    ctx.rc_controls.roll_stick_dps,
                    ctx.rc_controls.pitch_stick_dps,
                    ctx.rc_controls.yaw_stick_dps,
                    ctx.rc_controls.throttle_stick,
                ],
                pid_errors_s,
                pid_errors_k,
                rc_commands: ctx.rc_controls.controls_pwm,

                #[cfg(feature = "dshot_telemetry")]
                motor_rpm_d2: [0i16; SetpointMessage::MAX_SUPPORTED_MOTOR_COUNT],
                #[cfg(feature = "servos")]
                servos: [0i16; SetpointMessage::MAX_SUPPORTED_SERVO_COUNT],

                gps_state_flags: 0,
                failsafe_phase: 0,
                rx_signal_received: true,
                rx_flight_channel_is_valid: true,
            };
            ctx.setpoint_sender.send(setpoint_message);
        }
    }

    // ****
    // Check if there has been in-flight adjustment of the PID gains, if so apply them.
    // This happens infrequently.
    // ****
    //
    // try_next_message() is a simple pointer check. If there's no message, it returns None instantly,
    // so it won't mess up the 8kHz timing.
    if let Some(wait_result) = ctx.fast_config_subscriber.try_next_message()
        && let WaitResult::Message(fast_config_item) = wait_result
    {
        adjust_pid_gains(&mut ctx.flight_controller, fast_config_item);
    }

    Ok(())
}

fn adjust_pid_gains(flight_controller: &mut FlightController, fast_config_item: FastConfigItem) {
    match fast_config_item {
        FastConfigItem::RollRate(pid_config) => {
            let gains = FlightController::calculate_gains(pid_config);
            flight_controller.roll.rate_pid.set_gains(gains);
            flight_controller.roll.rate_pid.switch_integration_off();
            flight_controller.roll.rate_pid.set_setpoint(0.0);
        }
        FastConfigItem::PitchRate(pid_config) => {
            let gains = FlightController::calculate_gains(pid_config);
            flight_controller.pitch.rate_pid.set_gains(gains);
            flight_controller.pitch.rate_pid.switch_integration_off();
            flight_controller.pitch.rate_pid.set_setpoint(0.0);
        }
        FastConfigItem::YawRate(pid_config) => {
            let gains = FlightController::calculate_gains(pid_config);
            flight_controller.yaw_rate_pid.set_gains(gains);
            flight_controller.yaw_rate_pid.switch_integration_off();
            flight_controller.yaw_rate_pid.set_setpoint(0.0);
        }
        FastConfigItem::RollAngle(pid_config) => {
            let gains = FlightController::calculate_gains(pid_config);
            let pd_gains = PdGainsf32 { kp: gains.kp, kd: gains.kd };
            flight_controller.roll.angle_pid.set_gains(pd_gains);
            flight_controller.roll.angle_pid.set_setpoint(0.0);
        }
        FastConfigItem::PitchAngle(pid_config) => {
            let gains = FlightController::calculate_gains(pid_config);
            let pd_gains = PdGainsf32 { kp: gains.kp, kd: gains.kd };
            flight_controller.pitch.angle_pid.set_gains(pd_gains);
            flight_controller.pitch.angle_pid.set_setpoint(0.0);
        }
    }
}
