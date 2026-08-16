use imu_sensors::{ImuAxisOrder, ImuDevice};
use motor_mixers::MotorDriver;
use radio_controllers::{Radio, RadioType};

use crate::{
    barometer_sensors::{Barometer, BarometerType},
    gps::GpsProvider,
    magnetometer_sensors::{Magnetometer, MagnetometerType},
    optical_flow_sensors::{OpticalFlow, OpticalFlowType},
    rangefinder_sensors::{Rangefinder, RangefinderType},
};

#[cfg(not(feature = "rp2350"))]
use crate::{
    boards::platform::{GpsUartRx, GpsUartTx},
    gps::GpsParser,
};

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
    UartNotAvailable,
    UartError,
}

/// Parameters for `board_hardware`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardInit {
    pub axis_order: ImuAxisOrder,
    pub radio_type: RadioType,
    pub barometer_type: BarometerType,
    pub magnetometer_type: MagnetometerType,
    pub gps_provider: GpsProvider,
    pub rangefinder_type: RangefinderType,
    pub optical_flow_type: OpticalFlowType,
}

#[allow(unused)]
pub struct Board<I: ImuDevice> {
    pub imu: I,
    pub motor_driver: MotorDriver,
    //pub serial_rx_uart: Option<UartDevice>,
    pub radio: Radio,

    //pub max7456_spi: Option<SpiDeviceBlocking>,
    //pub sdcard_spi: Option<SpiDeviceAsync>,

    //pub msp_uart: Option<UartDevice>,
    //pub esc_sensor_uart: Option<UartDevice>,

    //pub sensors_i2c: Option<I2cDeviceBlocking>,
    pub barometer: Option<Barometer>,
    pub magnetometer: Option<Magnetometer>,
    pub gps: Option<GpsHardware>,
    pub rangefinder: Option<Rangefinder>,
    pub optical_flow: Option<OpticalFlow>,
}

#[allow(unused)]
#[cfg(not(feature = "rp2350"))]
pub struct GpsHardware {
    pub uart_rx: GpsUartRx,
    pub uart_tx: GpsUartTx,
    pub parser: GpsParser,
}
#[cfg(feature = "rp2350")]
pub struct GpsHardware {}
/// Context for IMU task.
pub struct ImuContext<I: ImuDevice> {
    pub imu: I,
}

impl<I: ImuDevice> ImuContext<I> {
    pub fn new(imu: I) -> Self {
        Self { imu }
    }
}
