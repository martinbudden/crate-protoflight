use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    watch::{Receiver, Sender, Watch},
};

use embassy_time::Instant;
use static_cell::StaticCell;

use imu_sensors::{AccFullScale, AccUnits, GyroFullScale, GyroUnits, ImuDevice};
use motor_mixers::MotorMixerMessage;
use sensor_fusion::{MadgwickFilterf32, SensorFusion};
use simple_bitset::BitSet64;

#[cfg(feature = "rpm_filters")]
use motor_mixers::RpmNotchFilterBankConfig;

use crate::{
    boards::BoardImu,
    config::{FastConfigItem, FastConfigSubscriber, fast_config_subscriber},
    flight::{FilterAccGyro, FlightController, ImuFilterBank, ImuFilterBankConfig, RcControls, VehicleControl},
    sensors::{GyroPidMessage, SetpointMessage},
    tasks::{
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
}

#[rustfmt::skip]
impl<I: ImuDevice> GyroPidContext<I> {
    pub fn new(
        imu: I,
        imu_filter_bank_config: ImuFilterBankConfig,
        #[cfg(feature = "rpm_filters")] rpm_notch_filter_bank_config: RpmNotchFilterBankConfig,
        #[cfg(feature = "rpm_filters")] looptime_seconds: f32,
    ) -> Self {
        Self {
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
            imu_filters: ImuFilterBank::with_config(
                imu_filter_bank_config,
            ),
            sensor_fusion: MadgwickFilterf32::new(),
            flight_controller: FlightController::new(),
            rc_controls: RcControls::new(),
            rc_modes: BitSet64::new(),
        }
    }
}

#[rustfmt::skip]
pub fn init(
    imu: BoardImu,
    imu_filter_bank_config: ImuFilterBankConfig,
    #[cfg(feature = "rpm_filters")] rpm_notch_filter_bank_config: RpmNotchFilterBankConfig,
    #[cfg(feature = "rpm_filters")] looptime_seconds: f32,
) -> &'static mut GyroPidContext<BoardImu> {
    GYRO_PID_CTX.init(GyroPidContext::new(
        imu,
        imu_filter_bank_config,
        #[cfg(feature = "rpm_filters")] rpm_notch_filter_bank_config,
        #[cfg(feature = "rpm_filters")] looptime_seconds,
    ))
}

/// The GYRO/PID task.
// The gyro_pid task calculates the motor commands, sends them immediately to the motor_mixer task
// and then updates the GyroPidMessage and sends it.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut GyroPidContext<BoardImu>) {
    log::info!("    GYRO_PID: task started");
    let mut loop_count: u32 = 0;
    let mut gyro_pid_send_count: u32 = 0;
    let gyro_pid_denominator = 10;

    let sample_rates = ctx.imu.init(8000, GyroFullScale::Max, GyroUnits::Rps, AccFullScale::Max, AccUnits::G).await;
    let (gyro_rate_hz, _acc_rate_hz) = match sample_rates {
        Ok((gyro_rate_hz, acc_rate_hz)) => (gyro_rate_hz, acc_rate_hz),
        Err(_err) => (1000, 1000),
    };

    #[allow(clippy::cast_precision_loss)]
    let delta_t: f32 = 1.0 / (gyro_rate_hz as f32);
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(u64::from(gyro_rate_hz)));

    // This is the famous GYRO/PID loop!
    loop {
        // ****
        // The GYRO part of the GYRO/PID loop
        // ****

        // TODO: this ticker wait should be replaced by an interrupt driven DMA read from the IMU.
        // I'm using a ticker like this during development to keep things simple.
        ticker.next().await;
        let time_us = Instant::now().as_micros();

        let acc_gyro = imu_sensors::Imu::read_acc_gyro(&mut ctx.imu).await;
        // let acc_gyro = imu_sensors::ImuDevice::read_acc_gyro(&mut ctx.imu).await;

        /*let (acc, gyro_rps) = match acc_gyro {
            Ok((acc, gyro_rps)) => (acc, gyro_rps),
            Err(_acc_gyro) => (Vector3f32::default(), Vector3f32::default()),
        };*/
        let (acc, gyro_rps) = acc_gyro.unwrap_or_default();

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

        // If there are new control values from the radio, then use them.
        if let Some(rx_message) = ctx.rx_receiver.try_changed() {
            ctx.rc_controls = rx_message.rc_controls;
            ctx.rc_modes = rx_message.rc_modes;
        }

        // Calculate the motor commands:
        // the flight controller updates its setpoints from the radio control_message
        // and then updates the PIDs using `gyro_rps` and `orientation`.
        // `setpoints_updated` is set if the setpoints have been updated because of a new radio_control_message,
        // or if the flight controller has updated the setpoints because of crash or spin recovery.
        let (motor_commands, setpoints_updated) = ctx.flight_controller.calculate_motor_commands(
            gyro_rps,
            orientation,
            delta_t,
            ctx.rc_controls,
            ctx.rc_modes,
        );

        // Convert the motor commands calculated by the flight controller into a motor mixer message and send that message.
        // The signal will be picked up by the motor mixer task.
        // We signal every time round the GYRO/PID loop since the motor mixer also updates the RPM notch filters on this signal.
        MOTOR_MIXER_SIGNAL.signal(MotorMixerMessage::from(motor_commands));

        // Send the GyroPidMessage on a denominator (e.g., 1/8 = 1kHz)
        // This will be picked up by the Blackbox, the OSD and anyone else who is listening.
        gyro_pid_send_count += 1;
        if gyro_pid_send_count >= gyro_pid_denominator {
            gyro_pid_send_count = 0;
            let gyro_pid_message =
                GyroPidMessage { orientation, acc, gyro_rps, gyro_rps_unfiltered, time_us, ..Default::default() };
            ctx.gyro_pid_sender.send(gyro_pid_message);
            if setpoints_updated {
                // Only send a setpoint_message when the setpoints have actually been updated
                // TODO: put the new setpoints in the setpoints message
                let mut setpoint_message = SetpointMessage::new();
                setpoint_message.time_us = time_us;
                setpoint_message.rc_modes = ctx.rc_modes;
                setpoint_message.setpoints = [
                    ctx.rc_controls.roll_stick_dps,
                    ctx.rc_controls.pitch_stick_dps,
                    ctx.rc_controls.yaw_stick_dps,
                    ctx.rc_controls.throttle_stick,
                ];
                setpoint_message.failsafe_phase = ctx.rc_controls.failsafe;
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
            && let embassy_sync::pubsub::WaitResult::Message(fast_config_item) = wait_result
        {
            match fast_config_item {
                FastConfigItem::RollRate(gains) => {
                    ctx.flight_controller.set_pid_gains(FlightController::ROLL_RATE_DPS, gains);
                }
                FastConfigItem::PitchRate(gains) => {
                    ctx.flight_controller.set_pid_gains(FlightController::PITCH_RATE_DPS, gains);
                }
                FastConfigItem::YawRate(gains) => {
                    ctx.flight_controller.set_pid_gains(FlightController::YAW_RATE_DPS, gains);
                }
                FastConfigItem::RollAngle(gains) => {
                    ctx.flight_controller.set_pid_gains(FlightController::ROLL_ANGLE_DEGREES, gains);
                }
                FastConfigItem::PitchAngle(gains) => {
                    ctx.flight_controller.set_pid_gains(FlightController::PITCH_ANGLE_DEGREES, gains);
                }
            }
        }

        if loop_count.is_multiple_of(1000) {
            log::info!("        GYRO_PID: loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
