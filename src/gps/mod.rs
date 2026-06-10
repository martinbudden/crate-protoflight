#![cfg(feature = "gps")]
#![doc = include_str!("README.md")]

mod config;

mod geodetic;
mod gps_data;
mod gps_solution_data;

pub use config::{GpsConfig, GpsRescueConfig};

pub use geodetic::{Geodetic, GeographicCoordinate};
pub use gps_data::{GpsData, GpsDataPosition, GpsMessage, GpsPosition, GpsYawHeadingMessage};
pub use gps_solution_data::{GpsSolutionData, GpsSolutionDataAbridged};
