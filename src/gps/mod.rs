#![doc = include_str!("README.md")]

mod config;
mod geodetic;
mod gps_solution_data;

#[cfg(feature = "gps")]
pub use config::{GpsConfig, GpsRescueConfig};
pub use geodetic::{Geodetic, GeographicCoordinate};
pub use gps_solution_data::{GpsSolutionData, GpsSolutionDataAbridged};
