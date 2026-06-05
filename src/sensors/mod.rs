#![doc = include_str!("README.md")]

mod barometer;
mod battery;
mod config;
mod rangefinder;

pub use battery::BatteryConfig;
pub use config::SensorConfig;

#[cfg(feature = "barometer")]
pub use barometer::BarometerData;
#[cfg(feature = "rangefinder")]
pub use rangefinder::RangefinderData;
