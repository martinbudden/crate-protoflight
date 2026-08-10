#![allow(unused)]
use cfg_if::cfg_if;

use imu_sensors::ImuDevice;
use motor_mixers::MotorDriver;

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
}

/// Context for IMU task.
pub struct ImuContext<I: ImuDevice> {
    pub imu: I,
}

impl<I: ImuDevice> ImuContext<I> {
    pub fn new(imu: I) -> Self {
        Self { imu }
    }
}

cfg_if! {
if #[cfg(feature = "stm32f405")] {

pub struct Board<I: ImuDevice> {
    pub imu: I,
    pub serial_rx_uart: Result<UartDevice, BoardInitError>,
    pub motor_driver: Result<MotorDriver, BoardInitError>,

    pub max7456_spi: Option<SpiDeviceBlocking>,
    pub sdcard_spi: Option<SpiDeviceAsync>,

    pub msp_uart: Option<UartDevice>,
    pub esc_sensor_uart: Option<UartDevice>,

    pub sensors_i2c: Option<I2cDeviceBlocking>,
}

// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking
use embassy_stm32::{
    gpio::{Output,Input},
    i2c::{I2c,mode::Master as I2cMaster},
    mode::{Async, Blocking},
    spi::{Spi, mode::Master as SpiMaster},
    usart::Uart,
};

pub type SpiDeviceAsync = embedded_hal_bus::spi::ExclusiveDevice<Spi<'static, Async, SpiMaster>, Output<'static>, embassy_time::Delay>;
pub type SpiDeviceBlocking = embedded_hal_bus::spi::ExclusiveDevice<Spi<'static, Blocking, SpiMaster>, Output<'static>, embassy_time::Delay>;
pub type I2cDeviceBlocking = I2c<'static, Blocking, I2cMaster>;
pub type UartDevice = Uart<'static, Async>;
pub type GpioInputPin = Input<'static>;
pub type GpioOutputPin = Output<'static>;
pub type MotorPins = [GpioOutputPin;8];

pub type MotorOutput = Output<'static>;

} else if #[cfg(feature = "rp2350")] {

pub struct Board<I: ImuDevice> {
    pub imu: I,
    pub serial_rx_uart: Result<UartDevice, BoardInitError>,
    pub motor_driver: Result<MotorDriver, BoardInitError>,

    pub sdcard_spi: Option<SdSpiDevice>,
    //pub osd_spi: AuxiliaryPioSpiDevice,

    pub msp_uart: Option<UartDevice>,
    pub sensors_i2c: Option<I2cDevice>,
}

// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking

// --- Device 1: Hardware SPI0 (Gyroscope) ---
// Tied to SPI0 running asynchronously via the DMA system
pub type GyroSpiBus = Spi<'static, peripherals::SPI0, SpiAsync>;
pub type GyroSpiDevice = ExclusiveDevice<GyroSpiBus, Output<'static>, Delay>;
pub type GpioInputPin = Input<'static>;
pub type GpioOutputPin = Output<'static>;
pub type MotorPins = [GpioOutputPin;8];

// --- Device 2: Hardware SPI1 (Blackbox SD Card) ---
// Tied to SPI1 running asynchronously via the DMA system
pub type SdSpiBus = Spi<'static, peripherals::SPI1, SpiAsync>;
pub type SdSpiDevice = ExclusiveDevice<SdSpiBus, Output<'static>, Delay>;

// TODO: placeholder
// --- Device 3: PIO0 Backed SPI (Auxiliary Peripheral - MAX7456) ---
// Fully concrete representation using State Machine 0 on the PIO0 block
//pub type AuxiliaryPioSpiDevice = ExclusiveDevice<PioSpi<'static, peripherals::PIO0, 0>, Output<'static>, Delay>;

pub type UartDevice = Uart<'static, UartAsync>;
pub type I2cDevice = I2c<'static, peripherals::I2C0, I2cAsync>;

use embassy_time::Delay;

use embedded_hal_bus::spi::ExclusiveDevice;

use embassy_rp::{
    Peri, bind_interrupts, dma, gpio,
    gpio::{Input, Level, Output, Pull},
    i2c,
    i2c::{Async as I2cAsync, I2c},
    peripherals, pio,
    pio::InterruptHandler as PioInterruptHandler,
    spi::{Async as SpiAsync, Spi},
    uart,
    uart::{Async as UartAsync, Uart},
};

} else  {

pub struct Board<I: ImuDevice> {
    pub imu: I,
    pub motor_driver: Result<MotorDriver, BoardInitError>,
}

}}
