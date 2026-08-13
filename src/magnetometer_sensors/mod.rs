#![doc = include_str!("README.md")]
#![allow(unused)]

mod config;
mod magnetometer;
mod magnetometer_hmc5883;
mod magnetometer_mock;

pub use config::MagnetometerConfig;
pub use magnetometer::{Magnetometer, MagnetometerMessage, MagnetometerType, RxMagnetometer};
