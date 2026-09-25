#![cfg(feature = "host")]
#![allow(unused)]

pub type RadioUartRx = ();
pub type RadioUartTx = ();

pub type GpsUartRx = ();
pub type GpsUartTx = ();

pub type I2cDeviceBlocking = crate::i2c_bus::MockI2c;
pub type SharedI2cBus = ();
