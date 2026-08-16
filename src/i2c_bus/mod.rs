#![doc = include_str!("README.md")]
#![allow(unused)]

mod i2c;
mod mock_i2c;

pub use i2c::{I2cError, SharedI2cBus, SharedI2cExt};

#[cfg(feature = "std")]
pub use mock_i2c::{MockI2c, MockI2cError};
