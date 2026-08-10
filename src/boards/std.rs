#![cfg(feature = "std")]
#![allow(unused)]
#![allow(clippy::similar_names)]

use crate::boards::{ImuContext, board::Board};
use imu_sensors::{Imu, ImuAxesOrder, ImuDevice, ImuMock, MockImuBus};
use motor_mixers::{MotorDriver, MotorDriverQuadPwm};

pub type BoardImu = ImuMock<MockImuBus>;

pub fn imu_context(imu: BoardImu) -> ImuContext<BoardImu> {
    ImuContext::new(imu)
}

pub fn board_init() -> Board<BoardImu> {
    let motor_driver_pwm = MotorDriverQuadPwm::new();
    let motor_driver = MotorDriver::QuadPwm(motor_driver_pwm);
    let imu = ImuMock::new(MockImuBus::new(), ImuAxesOrder::XPOS_YPOS_ZPOS);

    Board { imu, motor_driver: Ok(motor_driver) }
}
