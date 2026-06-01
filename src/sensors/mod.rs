#![doc = include_str!("README.md")]

mod barometer;
mod battery;
mod rangefinder;

#[cfg(feature = "barometer")]
pub use barometer::BarometerData;
pub use battery::BatteryConfig;
#[cfg(feature = "rangefinder")]
pub use rangefinder::RangefinderData;
