use super::{GpsUartRx, GpsUartTx, RadioUartRx, RadioUartTx};

use crate::barometer_sensors::{Barometer, BarometerType};
use crate::magnetometer_sensors::{Magnetometer, MagnetometerType};
use crate::optical_flow_sensors::{OpticalFlow, OpticalFlowType};
use crate::rangefinder_sensors::{Rangefinder, RangefinderType};

use imu_sensors::{ImuAxisOrder, ImuDevice};
use motor_mixers::{MotorDriver, MotorProtocol};

//#[cfg(all(feature = "rp2350xa", feature = "rp2350xb"))]
//compile_error!("rp2350xa and rp2350xb are mutually exclusive");

#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoardInitError {
    ImuNotAvailable,
    ImuError,
    GyroInterruptNotAvailable,
    GyroInterruptError,
    SdCardNotAvailable,
    SdCardError,
    Max7456NotAvailable,
    Max7456Error,
    SerialRxUartNotAvailable,
    SerialRxUartError,
    MspUartNotAvailable,
    MspUartError,
    EscSensorUartNotAvailable,
    EscSensorUartError,
    SensorsI2cNotAvailable,
    SensorsI2cError,
    MotorDriverNotAvailable,
    MotorDriverError,
    MotorProtocolNotSupported,
    UartNotAvailable,
    UartError,
}

/// Parameters for `board_hardware`.
#[allow(unused)]
pub struct BoardInit {
    pub spawner: embassy_executor::Spawner,
    pub axis_order: ImuAxisOrder,
    pub barometer_type: Option<BarometerType>,
    pub magnetometer_type: Option<MagnetometerType>,
    pub rangefinder_type: Option<RangefinderType>,
    pub optical_flow_type: Option<OpticalFlowType>,
    pub motor_protocol: MotorProtocol,
    pub motor_pwm_rate: u16,
    pub motor_pole_count: u8,
}

#[allow(unused)]
pub struct BoardHardware<I: ImuDevice> {
    pub imu: I,

    #[cfg(feature = "multicore")]
    pub gyro_pid_spawner: embassy_executor::SendSpawner,
    #[cfg(not(feature = "multicore"))]
    pub gyro_pid_spawner: embassy_executor::Spawner,
    #[cfg(feature = "realtime_executor")]
    pub realtime_spawner: embassy_executor::SendSpawner,
    #[cfg(not(feature = "realtime_executor"))]
    pub realtime_spawner: embassy_executor::Spawner,
    pub background_spawner: embassy_executor::Spawner,

    pub motor_driver: MotorDriver,

    pub radio_uart_rx: Option<RadioUartRx>,
    pub radio_uart_tx: Option<RadioUartTx>,

    pub gps_uart_rx: Option<GpsUartRx>,
    pub gps_uart_tx: Option<GpsUartTx>,

    //pub max7456_spi: Option<SpiDeviceBlocking>,
    //pub sdcard_spi: Option<SpiDeviceAsync>,

    //pub msp_uart: Option<UartDevice>,
    //pub esc_sensor_uart: Option<UartDevice>,
    pub barometer: Option<Barometer>,
    pub magnetometer: Option<Magnetometer>,
    pub rangefinder: Option<Rangefinder>,
    pub optical_flow: Option<OpticalFlow>,
}
