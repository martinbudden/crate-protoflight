#![cfg(feature = "std")]

use crate::{
    barometer_sensors::Barometer,
    boards::board::{Board, BoardInit, BoardInitError, ImuContext},
    gps::GpsParser,
    magnetometer_sensors::Magnetometer,
    optical_flow_sensors::OpticalFlow,
    rangefinder_sensors::Rangefinder,
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

    let barometer = Barometer::new(init.barometer_type);
    let magnetometer = Magnetometer::new(init.magnetometer_type);
    let gps_parser = GpsParser::new(init.gps_provider);
    let rangefinder = Rangefinder::new(init.rangefinder_type);
    let optical_flow = OpticalFlow::new(init.optical_flow_type);

    Ok(Board { imu, motor_driver, radio, barometer, magnetometer, gps_parser, rangefinder, optical_flow })
}
