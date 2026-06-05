#![doc = include_str!("README.md")]

mod barometer;
mod battery;
mod rangefinder;
mod sensor_flags;

pub use battery::BatteryConfig;
pub use sensor_flags::SensorFlags;

#[cfg(feature = "barometer")]
pub use barometer::BarometerData;
#[cfg(feature = "rangefinder")]
pub use rangefinder::RangefinderData;
