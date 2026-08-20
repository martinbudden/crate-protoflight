#![cfg(feature = "std")]

use crate::{
    barometer_sensors::Barometer,
    boards::board::{Board, BoardInit, BoardInitError, GpsHardware},
    boards::{GpsUartRx, GpsUartTx},
    i2c_bus::{MockI2c, SharedI2cBus},
    magnetometer_sensors::Magnetometer,
    optical_flow_sensors::OpticalFlow,
    rangefinder_sensors::Rangefinder,
};

use imu_sensors::{ImuMock, MockImuBus};
use motor_mixers::{MotorDriver, MotorDriverQuadPwm};
use radio_controllers::{Radio, RadioType};
use static_cell::StaticCell;

pub type BoardImu = ImuMock<MockImuBus>;

#[allow(clippy::unnecessary_wraps)]
pub fn board_hardware(init: BoardInit) -> Result<Board<BoardImu>, BoardInitError> {
    static I2C_BUS: StaticCell<SharedI2cBus> = StaticCell::new();

    let motor_driver_pwm = MotorDriverQuadPwm::new();
    let motor_driver = MotorDriver::QuadPwm(motor_driver_pwm);

    let imu = ImuMock::new(MockImuBus::new(), init.axis_order);

    let radio = Radio::new(RadioType::Mock);

    let shared_i2c = I2C_BUS.init(SharedI2cBus::new(MockI2c::new()));

    let barometer = Barometer::new(init.barometer_type, shared_i2c);
    let magnetometer = Magnetometer::new(init.magnetometer_type, shared_i2c);
    let gps_rx = GpsUartRx::default();
    let gps_tx = GpsUartTx::default();
    let gps = Some(GpsHardware { uart_rx: gps_rx, uart_tx: gps_tx });

    let rangefinder = Rangefinder::new(init.rangefinder_type);
    let optical_flow = OpticalFlow::new(init.optical_flow_type);

    Ok(Board { imu, motor_driver, radio, barometer, magnetometer, gps, rangefinder, optical_flow })
}
