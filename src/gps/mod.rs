#![cfg(feature = "gps")]
#![doc = include_str!("README.md")]

mod config;

mod geodetic;
mod gps_data;
mod gps_solution_data;

pub use config::{GpsConfig, GpsProvider, GpsRescueConfig, SbasMode};

pub use geodetic::{Geodetic, GeographicCoordinate};
pub use gps_data::{GpsData, GpsMessage, GpsPositionLongLatAlt, GpsPositionMeters, GpsYawHeadingMessage};
pub use gps_solution_data::{GpsSolutionData, GpsSolutionDataAbridged};
