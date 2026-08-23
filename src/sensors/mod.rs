#![doc = include_str!("README.md")]

mod battery;
mod current_sensor;
mod sensor_flags;

pub use current_sensor::{CurrentSensorAdcConfig, CurrentSensorVirtualConfig};
pub use sensor_flags::SensorFlags;

#[cfg(feature = "battery")]
pub use battery::{BatteryConfig, BatteryMessage, BatteryProfiles, CurrentMeterReading, VoltageMeterReading};
