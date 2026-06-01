#![cfg(feature = "gps")]
#![doc = include_str!("README.md")]

mod config;
mod geodetic;
mod gps_data;
mod gps_sensor_data;
mod gps_solution_data;

pub use config::{GpsConfig, GpsRescueConfig};
pub use geodetic::{Geodetic, GeographicCoordinate};
pub use gps_data::{GpsData, GpsDataPosition, GpsPosition, GpsYawHeadingData};
pub use gps_sensor_data::{
    GpsDataItem, GpsDataPublisher, GpsDataSubscriber, YawHeadingPublisher, YawHeadingSubscriber, gps_data_publisher,
    gps_data_subscriber, yaw_heading_publisher, yaw_heading_subscriber,
};
pub use gps_solution_data::{GpsSolutionData, GpsSolutionDataAbridged};
