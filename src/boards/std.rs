#![cfg(feature = "std")]

use crate::{
    barometer_sensors::{Barometer, BarometerType},
    boards::board::{Board, BoardInit, BoardInitError, ImuContext},
    magnetometer_sensors::{Magnetometer, MagnetometerType},
    optical_flow_sensors::{OpticalFlow, OpticalFlowType},
    rangefinder_sensors::{Rangefinder, RangefinderType},
};

use imu_sensors::{ImuMock, MockImuBus};
use motor_mixers::{MotorDriver, MotorDriverQuadPwm};
use radio_controllers::{Radio, RadioType};

pub type BoardImu = ImuMock<MockImuBus>;

#[allow(unused)]
pub fn imu_context(imu: BoardImu) -> ImuContext<BoardImu> {
    ImuContext::new(imu)
}

#[allow(clippy::unnecessary_wraps)]
pub fn board_init(init: BoardInit) -> Result<Board<BoardImu>, BoardInitError> {
    let motor_driver_pwm = MotorDriverQuadPwm::new();
    let motor_driver = MotorDriver::QuadPwm(motor_driver_pwm);

    let imu = ImuMock::new(MockImuBus::new(), init.axis_order);

    let radio = Radio::new(RadioType::Mock);

    let barometer = Barometer::new(BarometerType::Mock);

    let magnetometer = Magnetometer::new(MagnetometerType::Mock);

    let rangefinder = Rangefinder::new(RangefinderType::Mock);

    let optical_flow = OpticalFlow::new(OpticalFlowType::Mock);

    Ok(Board { imu, motor_driver, radio, barometer, magnetometer, rangefinder, optical_flow })
}
