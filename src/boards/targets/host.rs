#![cfg(feature = "host")]

use crate::boards::board::{BoardHardware, BoardInit, BoardInitError};

use imu_sensors::{ImuMock, MockImuBus};
use motor_mixers::{MotorDriver, MotorDriverPwm};

pub type BoardImu = ImuMock<MockImuBus>;
pub type Board = BoardHardware<BoardImu>;

impl Board {
    #[allow(clippy::unnecessary_wraps)]
    pub fn new(init: &BoardInit) -> Result<Self, BoardInitError> {
        let imu = ImuMock::new(MockImuBus::new(), init.axis_order);

        let motor_driver_pwm = MotorDriverPwm::new();
        let motor_driver = MotorDriver::Pwm(motor_driver_pwm);

        let radio_uart_tx = None;
        let radio_uart_rx = None;

        let gps_uart_tx = None;
        let gps_uart_rx = None;

        let barometer = None;
        let magnetometer = None;
        let rangefinder = None;
        let optical_flow = None;

        Ok(Self {
            gyro_pid_spawner: init.spawner,
            realtime_spawner: init.spawner,
            background_spawner: init.spawner,
            imu,
            motor_driver,
            radio_uart_rx,
            radio_uart_tx,
            gps_uart_rx,
            gps_uart_tx,
            barometer,
            magnetometer,
            rangefinder,
            optical_flow,
        })
    }
}
