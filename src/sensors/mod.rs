#![doc = include_str!("README.md")]

mod barometer;
mod battery;
mod magnetometer;
mod messages;
mod optical_flow;
mod rangefinder;
mod sensor_flags;

pub use messages::{GyroPidMessage, SetpointMessage};
pub use sensor_flags::SensorFlags;

#[cfg(feature = "barometer")]
pub use barometer::{BarometerConfig, BarometerMessage};
#[cfg(feature = "battery")]
pub use battery::{BatteryConfig, BatteryMessage, BatteryProfiles, CurrentMeterReading, VoltageMeterReading};
#[cfg(feature = "magnetometer")]
pub use magnetometer::MagnetometerConfig;
#[cfg(feature = "optical_flow")]
pub use optical_flow::{OpticalFlowConfig, OpticalFlowMessage};
#[cfg(feature = "rangefinder")]
pub use rangefinder::{RangefinderConfig, RangefinderMessage};
