use blackbox_logger::{GyroPidMessage, SetpointMessage};
use embassy_time::{Duration, Timer};
use imu_sensors::{Imu, ImuCommon, ImuMock, MockImuBus};
use log::info;
use motor_mixers::MotorMixerMessage;
use rand::RngExt;
use sensor_fusion::{MadgwickFilterf32, SensorFusion};
use static_cell::StaticCell;
use vqm::{Vector3df32, Vector3di32};

use crate::{
    config::{GyroPidItem, GyroPidSubscriber},
    dispatch::{GyroPidMessageSender, SetpointMessageSender},
    flight::{FilterAccGyro, FlightController, ImuFilterBank, VehicleControl},
    tasks::{motor_mixer_task::MOTOR_MIXER_SIGNAL, radio_task::RadioReceiver},
};

#[cfg(feature = "rp2350")]
use embassy_rp::gpio::{Input, Pull};
#[cfg(feature = "rp2350")]
use embassy_rp::interrupt;
#[cfg(feature = "rp2350")]
use embassy_rp::interrupt::{InterruptExt, Priority};

#[cfg(feature = "multicore")]
use embassy_executor::InterruptExecutor;
#[cfg(feature = "multicore")]
// TODO: put EXECUTOR_CORE1 in a static cell
static EXECUTOR_CORE1: InterruptExecutor = InterruptExecutor::new();
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

pub(crate) static GYRO_CTX: StaticCell<GyroPidContext> = StaticCell::new();

/// Context for gyro_pid_task.
pub struct GyroPidContext<'a> {
    pub radio_receiver: RadioReceiver,
    pub gyro_pid_sender: GyroPidMessageSender,
    pub setpoint_sender: SetpointMessageSender,
    pub gyro_pid_subscriber: GyroPidSubscriber<'a>,
    pub imu: ImuMock<MockImuBus>,
    pub imu_filters: ImuFilterBank,
    pub sensor_fusion: MadgwickFilterf32,
    pub flight_controller: FlightController,
}

/// The GYRO/PID task.
#[embassy_executor::task]
pub async fn gyro_pid_task(ctx: &'static mut GyroPidContext<'static>) {
    info!(" GYRO_PID: task started");
    let mut time_us: u32 = 0;
    let mut loop_count: u32 = 0;
    let mut gyro_pid_send_count: u32 = 0;
    let gyro_pid_denominator = 10;
    let mut my_rng = rand::rng();
    // Base signal levels
    let mut x_base: i32 = 0;
    let delta_t = 0.0001;

    let _ = ctx.imu.init(8000, ImuCommon::GYRO_FULL_SCALE_MAX, ImuCommon::ACC_FULL_SCALE_MAX).await;

    // This is the famous GYRO/PID loop!
    loop {
        // Drain all pending messages to get to the latest state
        // try_next_message() is a simple pointer check. If there's no message, it returns None instantly,
        // so it won't mess up the 8kHz timing.

        // check if there has been in-flight adjustment of the PID gains, if so apply them.
        while let Some(wait_result) = ctx.gyro_pid_subscriber.try_next_message() {
            if let embassy_sync::pubsub::WaitResult::Message(event) = wait_result {
                match event {
                    GyroPidItem::RollRate(gains) => {
                        ctx.flight_controller.set_pid_gains(FlightController::ROLL_RATE_DPS, gains);
                    }
                    GyroPidItem::PitchRate(gains) => {
                        ctx.flight_controller.set_pid_gains(FlightController::PITCH_RATE_DPS, gains);
                    }
                    GyroPidItem::YawRate(gains) => {
                        ctx.flight_controller.set_pid_gains(FlightController::YAW_RATE_DPS, gains);
                    }
                    GyroPidItem::RollAngle(gains) => {
                        ctx.flight_controller.set_pid_gains(FlightController::ROLL_ANGLE_DEGREES, gains);
                    }
                    GyroPidItem::PitchAngle(gains) => {
                        ctx.flight_controller.set_pid_gains(FlightController::PITCH_ANGLE_DEGREES, gains);
                    }
                }
            }
        }

        //
        // The GYRO part of the GYRO/PID loop
        //

        // For now we are just faking some gyro and acc values.
        let acc_rnd = Vector3df32 { x: 1.0, y: 0.5, z: 0.25 };
        ctx.imu.set_acc(acc_rnd).await;
        x_base += my_rng.random_range(-5..=5);
        let gyro_raw = Vector3di32 {
            x: x_base + my_rng.random_range(-2..=2),
            y: my_rng.random_range(-5..=5),
            z: my_rng.random_range(-5..=5),
        };
        let gyro_dps_rnd = Vector3df32::from(gyro_raw);
        ctx.imu.set_gyro_dps(gyro_dps_rnd).await;

        // ctx.drdy.wait_for_rising_edge().await; // Synchronized to IMU
        // let data = read_imu_dma(&mut ctx.spi).await;
        /*let (acc, gyro_rps) = match ctx.imu.read_acc_gyro_rps().await {
            Ok(acc) => acc,
            Err(e) => (Vector3df32::default(),Vector3df32::default()),
        };*/
        let (acc, gyro_rps) = ctx.imu.read_acc_gyro_rps().await.unwrap_or_default();

        // Save the unfiltered gyro value for telemetry.
        let gyro_rps_unfiltered = gyro_rps;

        // Filter the acc and gyro values. This includes RPM notch filtering, if that is enabled.
        let (acc, gyro_rps) = ctx.imu_filters.update(acc, gyro_rps, delta_t);

        // Calculate the orientation quaternion using sensor fusion.
        let orientation = ctx.sensor_fusion.fuse_acc_gyro(acc, gyro_rps, delta_t);

        //
        // The PID part of the GYRO/PID loop
        //

        // get(peek) the latest radio control message - this is a non-blocking wait.
        let radio_control_message = ctx.radio_receiver.get().await;

        // Calculate the motor commands:
        // the flight controller updates its setpoints from the radio control_message
        // and the updates the PIDs using `gyro_rps` and `orientation`.
        // Also returns if the setpoints have been updated because of a new radio_control_message.
        let (motor_commands, setpoints_updated) =
            ctx.flight_controller.calculate_motor_commands(gyro_rps, orientation, delta_t, radio_control_message);

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
                let setpoint_message = SetpointMessage::new();
                ctx.setpoint_sender.send(setpoint_message);
            }
        }

        // Increment fake time (e.g., 1000us per sample for 1kHz)
        time_us = time_us.wrapping_add(125); // use wrapping_add to handle when time rolls over at max u32.

        /*if time_us.is_multiple_of(10000) {
            info!("GYRO_PID: time {time_us}");
        }*/
        if loop_count.is_multiple_of(100) {
            info!(" GYRO_PID: loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.

        // Slow down the simulation for PC console
        // 100ms is good for seeing the prints; change to 1ms for "real speed".
        Timer::after(Duration::from_millis(1)).await;
    }
}
