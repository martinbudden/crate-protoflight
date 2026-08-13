#![doc = include_str!("README.md")]
#![allow(unused)]

mod barometer;
mod barometer_bmp085;
mod barometer_mock;
mod config;

pub use barometer::{Barometer, BarometerDevice, BarometerMessage, BarometerType};
pub use config::BarometerConfig;
