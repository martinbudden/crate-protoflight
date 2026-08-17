#![cfg(feature = "stm32")]

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking
use embassy_stm32::{
    gpio::{Input, Output},
    i2c::{I2c, mode::Master as I2cMaster},
    mode::{Async, Blocking},
    spi::{Spi, mode::Master as SpiMaster},
    usart::Uart,
};

pub type SpiDeviceAsync =
    embedded_hal_bus::spi::ExclusiveDevice<Spi<'static, Async, SpiMaster>, Output<'static>, embassy_time::Delay>;
pub type SpiDeviceBlocking =
    embedded_hal_bus::spi::ExclusiveDevice<Spi<'static, Blocking, SpiMaster>, Output<'static>, embassy_time::Delay>;
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
pub type MotorPins = [GpioOutputPin; 8];

pub type MotorOutput = Output<'static>;
pub type SharedI2cBus = Mutex<NoopRawMutex, I2cDeviceBlocking>;
