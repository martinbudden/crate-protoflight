#![cfg(feature = "std")]

use crate::boards::{ImuContext, board::{Board, BoardInitError}};
use imu_sensors::{ImuAxisOrder, ImuMock, MockImuBus};
use motor_mixers::{MotorDriver, MotorDriverQuadPwm};

pub type BoardImu = ImuMock<MockImuBus>;

#[allow(unused)]
pub fn imu_context(imu: BoardImu) -> ImuContext<BoardImu> {
    ImuContext::new(imu)
}

#[allow(clippy::unnecessary_wraps)]
pub fn board_init(axis_order: ImuAxisOrder) -> Result<Board<BoardImu>, BoardInitError> {
    let motor_driver_pwm = MotorDriverQuadPwm::new();
    let motor_driver = MotorDriver::QuadPwm(motor_driver_pwm);
    let imu = ImuMock::new(MockImuBus::new(), axis_order);

    Ok(Board { imu, motor_driver })
}
