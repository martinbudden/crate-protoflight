#![doc = include_str!("README.md")]

mod barometer;
mod battery;
mod magnetometer;
mod optical_flow;
mod rangefinder;
mod sensor_flags;

pub use sensor_flags::SensorFlags;

#[cfg(feature = "barometer")]
pub use barometer::{BarometerConfig, BarometerData};
#[cfg(feature = "battery")]
pub use battery::{BatteryConfig, BatteryData, BatteryProfiles, CurrentMeterReading, VoltageMeterReading};
#[cfg(feature = "magnetometer")]
pub use magnetometer::MagnetometerConfig;
#[cfg(feature = "optical_flow")]
pub use optical_flow::OpticalFlowConfig;
#[cfg(feature = "rangefinder")]
pub use rangefinder::{RangefinderConfig, RangefinderData};
