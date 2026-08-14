#![allow(unused)]
use cfg_if::cfg_if;

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use imu_sensors::{ImuAxisOrder, ImuDevice};
use motor_mixers::MotorDriver;
use radio_controllers::{Radio, RadioType};
#[cfg(feature = "stm32")]
use static_cell::StaticCell;

use crate::{
    barometer_sensors::{Barometer, BarometerType},
    gps::{GpsParser, GpsProvider},
    magnetometer_sensors::{Magnetometer, MagnetometerType},
    optical_flow_sensors::{OpticalFlow, OpticalFlowType},
    rangefinder_sensors::{Rangefinder, RangefinderType},
};
#[cfg(feature = "stm32")]
use embassy_stm32::interrupt::{self, InterruptExt, Priority};

//#[cfg(all(feature = "rp2350xa", feature = "rp2350xb"))]
//compile_error!("rp2350xa and rp2350xb are mutually exclusive");

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

pub struct GpsHardware {
    pub uart_rx: GpsUartRx,
    pub uart_tx: GpsUartTx,
    pub parser: GpsParser,
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
pub type I2cDeviceAsync = I2c<'static, Async, I2cMaster>;
pub type UartDevice = Uart<'static, Async>;
//pub type GpsUartRx = UartDevice;
//pub type GpsUartTx = UartDevice;
pub type GpsUartRx = embassy_stm32::usart::UartRx<'static, Async>;
pub type GpsUartTx = embassy_stm32::usart::UartTx<'static, Async>;
//pub type TargetUart = embassy_stm32::usart::Uart<'static, embassy_stm32::mode::Async>;

pub type GpioInputPin = Input<'static>;
pub type GpioOutputPin = Output<'static>;
pub type MotorPins = [GpioOutputPin;8];

pub type MotorOutput = Output<'static>;
pub type SharedI2cBus = Mutex<NoopRawMutex, I2cDeviceBlocking>;


} else if #[cfg(feature = "rp2350")] {

// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking

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

//pub type I2cDevice0Async = I2c<'static, peripherals::I2C0, I2cAsync>;
pub type I2cDeviceAsync = I2c<'static, peripherals::I2C0, I2cAsync>;
//pub type I2cDevice0Blocking = I2c<'static, peripherals::I2C0, I2cBlocking>;
pub type SharedI2cBus = embassy_sync::mutex::Mutex< NoopRawMutex, I2cDeviceAsync>;

//pub type UartDevice = Uart<'static, UartAsync>;
pub type UartDevice0 = embassy_rp::uart::UartRx<'static, embassy_rp::peripherals::UART0, embassy_rp::uart::Async>;
pub type UartDevice1 = embassy_rp::uart::UartRx<'static, embassy_rp::peripherals::UART1, embassy_rp::uart::Async>;
pub type GpsUartRx =
    embassy_rp::uart::UartRx<
        'static,
        embassy_rp::peripherals::UART0,
        embassy_rp::uart::Async,
    >;

pub type GpsUartTx =
    embassy_rp::uart::UartTx<
        'static,
        embassy_rp::peripherals::UART0,
        embassy_rp::uart::Async,
    >;pub type RawI2c = embassy_rp::i2c::I2c<'static, embassy_rp::peripherals::I2C0, embassy_rp::i2c::Async>;

} else  {

use crate::boards::{mock_i2c::MockI2c, mock_uart::MockUart};

pub type GpsUartRx = MockUart;
pub type GpsUartTx = MockUart;
pub type RawI2c = MockI2c; // Reuse your mock on host
pub type SharedI2cBus = Mutex<NoopRawMutex, MockI2c>;
pub type I2cDeviceBlocking = MockI2c;

}}
