#![doc = include_str!("README.md")]

mod barometer;
mod barometer_bmp085;
mod barometer_device;
mod barometer_dps310;
mod barometer_mock;
mod config;

pub use barometer::{Barometer, BarometerI2cError, BarometerType};
pub use barometer_device::{BarometerDevice, BarometerMessage};
#[allow(unused)]
pub use config::BarometerConfig;
