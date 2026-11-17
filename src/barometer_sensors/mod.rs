#![doc = include_str!("README.md")]

mod barometer;
mod barometer_bmp280;
mod barometer_device;
mod barometer_dps310;
mod barometer_mock;
mod config;

pub use barometer::{Barometer, BarometerType};
pub use barometer_device::{BarometerDevice, BarometerI2cError, BarometerMessage};

#[cfg(feature = "barometer")]
pub use config::BarometerConfig;
