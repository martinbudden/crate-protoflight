#![cfg(feature = "std")]

use crate::boards::{ImuContext, board::Board};
use imu_sensors::{ImuAxisOrder, ImuMock, MockImuBus};
use motor_mixers::{MotorDriver, MotorDriverQuadPwm};

pub type BoardImu = ImuMock<MockImuBus>;

#[allow(unused)]
pub fn imu_context(imu: BoardImu) -> ImuContext<BoardImu> {
    ImuContext::new(imu)
}

pub fn board_init(axis_order: ImuAxisOrder) -> Board<BoardImu> {
    let motor_driver_pwm = MotorDriverQuadPwm::new();
    let motor_driver = MotorDriver::QuadPwm(motor_driver_pwm);
    let imu = ImuMock::new(MockImuBus::new(), axis_order);

    Board { imu, motor_driver: Ok(motor_driver) }
}
