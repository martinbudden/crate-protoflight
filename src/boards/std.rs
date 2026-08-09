#![cfg(feature = "std")]
#![allow(unused)]
#![allow(clippy::similar_names)]

use crate::boards::{ImuContext, board::Board};
use imu_sensors::{Imu, ImuDevice, ImuMock, MockImuBus};
use motor_mixers::{MotorDriver, MotorDriverQuadPwm};
use static_cell::StaticCell;

pub type BoardImu = ImuMock<MockImuBus>;

static IMU_CTX: StaticCell<ImuContext<BoardImu>> = StaticCell::new();

pub fn imu_context(imu: BoardImu) -> &'static mut ImuContext<BoardImu> {
    IMU_CTX.init(ImuContext::new(imu))
}

pub fn init() -> Board<BoardImu> {
    let motor_driver_pwm = MotorDriverQuadPwm::new();
    let motor_driver = MotorDriver::QuadPwm(motor_driver_pwm);
    let imu = ImuMock::new(MockImuBus::new(), imu_sensors::ImuAxesOrder::XPOS_YPOS_ZPOS);

    Board { imu, motor_driver: Ok(motor_driver) }
}
