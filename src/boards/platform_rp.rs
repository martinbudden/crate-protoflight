#![cfg(any(feature = "rp2040", feature = "rp235xa", feature = "rp235xb"))]

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking

use embassy_rp::{
    i2c::{Blocking, I2c},
    peripherals,
    uart::{Async, UartRx, UartTx},
};

pub type I2cDeviceBlocking = I2c<'static, peripherals::I2C0, Blocking>;
pub type SharedI2cBus = Mutex<NoopRawMutex, I2cDeviceBlocking>;
//pub type I2cDevice0Async = I2c<'static, peripherals::I2C0, Async>;
//pub type AuxiliaryPioSpiDevice = ExclusiveDevice<PioSpi<'static, peripherals::PIO0, 0>, Output<'static>, Delay>;

pub type RadioUartRx = &'static mut UartRx<'static, Async>;
pub type RadioUartTx = &'static mut UartTx<'static, Async>;

pub type GpsUartRx = &'static mut UartRx<'static, Async>;
pub type GpsUartTx = &'static mut UartTx<'static, Async>;
