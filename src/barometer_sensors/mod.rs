#![doc = include_str!("README.md")]
#![allow(unused)]

mod barometer;
mod barometer_bmp085;
mod barometer_mock;
mod config;

pub use barometer::{Barometer, BarometerMessage, BarometerType, RxBarometer};
pub use config::BarometerConfig;
