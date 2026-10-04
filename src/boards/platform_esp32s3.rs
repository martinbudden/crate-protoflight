#![cfg(feature = "esp32s3")]
#![allow(unused)]

pub type I2cDeviceBlocking = crate::i2c_bus::MockI2c;
pub type SharedI2cBus = ();

pub type RadioUartRx = ();
pub type RadioUartTx = ();

pub type GpsUartRx = ();
pub type GpsUartTx = ();
