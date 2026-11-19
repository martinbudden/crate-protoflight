#![doc = include_str!("README.md")]

mod barometer;
mod barometer_message;
mod config;

pub use barometer::{Barometer, BarometerType};
pub use barometer_message::BarometerMessage;

#[cfg(feature = "barometer")]
pub use config::BarometerConfig;
