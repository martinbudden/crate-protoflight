use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use log::info;
use tinyrand::{RandRange, StdRand};

use imu_sensors::{ImuCommon, ImuMock, MockImuBus};
use vqm::{Vector3df32, Vector3di32};

#[cfg(feature = "rp2350")]
use embassy_rp::{
    gpio::{Input, Pull},
    interrupt::{self, InterruptExt, Priority},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImuData {
    pub acc: Vector3df32,
    pub gyro_rps: Vector3df32,
    pub delta_t: f32,
}

impl ImuData {
    pub const fn new() -> Self {
        Self {
            acc: Vector3df32 { x: 0.0, y: 0.0, z: 0.0 },
            gyro_rps: Vector3df32 { x: 0.0, y: 0.0, z: 0.0 },
            delta_t: 0.1,
        }
    }
}

impl Default for ImuData {
    fn default() -> Self {
        Self::new()
    }
}

pub static IMU_SIGNAL: Signal<CriticalSectionRawMutex, ImuData> = Signal::new();

/// Context for IMU task.
pub struct ImuContext {
    pub imu: ImuMock<MockImuBus>,
}

/// IMU Task Placeholder.
#[embassy_executor::task]
pub async fn imu_task(ctx: &'static mut ImuContext) {
    let delta_t_us = 1000;
    let delta_t = 0.001_f32;
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_micros(delta_t_us));
    let mut loop_count: u32 = 0;
    let mut rand = StdRand::default();
    // Base signal levels
    let mut x_base: i32 = 0;

    let _ = ctx.imu.init(8000, ImuCommon::GYRO_FULL_SCALE_MAX, ImuCommon::ACC_FULL_SCALE_MAX).await;
    info!("      IMU: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;

        // For now we are just faking some gyro and acc values.
        let acc_rnd = Vector3df32 { x: 1.0, y: 0.5, z: 0.25 };
        ctx.imu.set_acc(acc_rnd).await;
        x_base += rand.next_range(0..5_u32).cast_signed() - 2;
        let gyro_raw = Vector3di32 {
            x: x_base + rand.next_range(0..5_u32).cast_signed() - 2,
            y: rand.next_range(0..11_u32).cast_signed() - 5,
            z: rand.next_range(0..11_u32).cast_signed() - 5,
        };
        let gyro_dps_rnd = Vector3df32::from(gyro_raw);
        ctx.imu.set_gyro_dps(gyro_dps_rnd).await;

        // ctx.drdy.wait_for_rising_edge().await; // Synchronized to IMU
        // let data = read_imu_dma(&mut ctx.spi).await;
        /*let (acc, gyro_rps) = match ctx.imu.read_acc_gyro_rps().await {
            Ok(acc) => acc,
            Err(e) => (Vector3df32::default(),Vector3df32::default()),
        };*/
        //let (acc, gyro_rps) = ctx.imu.read_acc_gyro_rps().await.unwrap_or_default();

        // Signal the gyro_pid task that there is new ImuData available.
        let imu_data = ImuData { acc: acc_rnd, gyro_rps: gyro_dps_rnd.to_radians(), delta_t };
        IMU_SIGNAL.signal(imu_data);

        if loop_count.is_multiple_of(100) {
            info!("           IMU:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.

        // Slow down the simulation for PC console
        // 100ms is good for seeing the prints; change to 1ms for "real speed".
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1)).await;
    }
}
