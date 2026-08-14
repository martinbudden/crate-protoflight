#![doc = include_str!("README.md")]
#![allow(unused)]

mod barometer;
mod barometer_bmp085;
mod barometer_device;
mod barometer_dps310;
mod barometer_mock;
mod config;
mod i2c;

pub use barometer::{Barometer, BarometerType};
pub use barometer_device::{BarometerDevice, BarometerMessage};
pub use config::BarometerConfig;
