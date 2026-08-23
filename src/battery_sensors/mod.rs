#![doc = include_str!("README.md")]

mod battery;
mod current_sensor;

pub use current_sensor::{CurrentSensorAdcConfig, CurrentSensorVirtualConfig};

#[cfg(feature = "battery")]
pub use battery::{BatteryConfig, BatteryMessage, BatteryProfiles, CurrentMeterReading, VoltageMeterReading};
