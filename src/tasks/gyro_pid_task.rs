use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    watch::{Receiver, Sender, Watch},
};

use motor_mixers::MotorMixerMessage;
use sensor_fusion::{MadgwickFilterf32, SensorFusion};

use crate::{
    config::{FastConfigItem, FastConfigSubscriber},
    flight::{FilterAccGyro, FlightControlMessage, FlightController, ImuFilterBank, VehicleControl},
    sensors::{GyroPidMessage, SetpointMessage},
    tasks::{imu_task::IMU_SIGNAL, motor_mixer_task::MOTOR_MIXER_SIGNAL, rx_task::FlightControlReceiver},
};

#[cfg(feature = "gps")]
use crate::tasks::gps_task::GPS_YAW_HEADING_SIGNAL;

#[cfg(feature = "multicore")]
// TODO: put EXECUTOR_CORE1 in a static cell
static EXECUTOR_CORE1: embassy_executor::InterruptExecutor = InterruptExecutor::new();
//static EXECUTOR_CORE1: StaticCell<Executor> = StaticCell::new();

/// Spawns `gyro_pid_task` to core1 if we are using a dual-core processor.
#[cfg(feature = "multicore")]
fn core1_entry(ctx_ptr: usize) -> ! {
    // 1. Retrieve the context pointer passed from Core 0
    let ctx = unsafe { &mut *(ctx_ptr as *mut GyroPidContext) };

    let spawner = EXECUTOR_CORE1.start(interrupt::IO_IRQ_BANK0);
    spawner.spawn(gyro_pid_task(ctx)).unwrap();

    loop {
        cortex_m::asm::wfi();
    }
}

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

/// Context for `gyro_pid_task`.
pub struct GyroPidContext<'a> {
    pub flight_control_receiver: FlightControlReceiver,
    pub gyro_pid_sender: GyroPidSender,
    pub setpoint_sender: SetpointSender,
    pub fast_config_subscriber: FastConfigSubscriber<'a>,
    pub imu_filters: ImuFilterBank,
    pub sensor_fusion: MadgwickFilterf32,
    pub flight_controller: FlightController,
    pub flight_control_message: FlightControlMessage,
}

/// The GYRO/PID task.
#[embassy_executor::task]
pub async fn gyro_pid_task(ctx: &'static mut GyroPidContext<'static>) {
    log::info!(" GYRO_PID: task started");
    let mut time_us: u32 = 0;
    let mut loop_count: u32 = 0;
    let mut gyro_pid_send_count: u32 = 0;
    let gyro_pid_denominator = 10;

    // This is the famous GYRO/PID loop!
    loop {
        // ****
        // The GYRO part of the GYRO/PID loop
        // ****

        // TODO: this signal should be replaced by an interrupt driven DMA read from the IMU.
        // I'm using a Signal like this during development to keep things simple.
        let imu_data = IMU_SIGNAL.wait().await;
        let delta_t = imu_data.delta_t;

        // Save the unfiltered gyro value for telemetry.
        let gyro_rps_unfiltered = imu_data.gyro_rps;

        // Filter the acc and gyro values. This includes RPM notch filtering, if that is enabled.
        let (acc, gyro_rps) = ctx.imu_filters.update(imu_data.acc, imu_data.gyro_rps, delta_t);

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
        if let Some(flight_control_message) = ctx.flight_control_receiver.try_changed() {
            ctx.flight_control_message = flight_control_message;
        }

        // Calculate the motor commands:
        // the flight controller updates its setpoints from the radio control_message
        // and then updates the PIDs using `gyro_rps` and `orientation`.
        // `setpoints_updated` is set if the setpoints have been updated because of a new radio_control_message.
        let (motor_commands, setpoints_updated) =
            ctx.flight_controller.calculate_motor_commands(gyro_rps, orientation, delta_t, ctx.flight_control_message);

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
                GyroPidMessage { acc, gyro_rps, gyro_rps_unfiltered, orientation, time_us, ..Default::default() };
            ctx.gyro_pid_sender.send(gyro_pid_message);
            if setpoints_updated {
                // Only send a setpoint_message when the setpoints have actually been updated
                // TODO: put the new setpoints in the setpoints message
                let mut setpoint_message = SetpointMessage::new();
                setpoint_message.time_us = time_us;
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

        // Increment fake time (e.g., 1000us per sample for 1kHz)
        time_us = time_us.wrapping_add(125); // use wrapping_add to handle when time rolls over at max u32.

        if loop_count.is_multiple_of(100) {
            log::info!("      GYRO_PID: loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
