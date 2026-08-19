#![doc = include_str!("README.md")]
#![allow(unused)]

mod config;

mod geodetic;
mod gps_data;
mod gps_message;
mod gps_parser;
mod gps_solution_data;
mod nmea_gga;
mod nmea_gsa;
mod nmea_gsv;
mod nmea_parser;
mod nmea_rmc;
mod ubx_nav_pvt_data;
mod ubx_parser;

pub use config::{GpsConfig, GpsOffOn, GpsProvider, GpsRescueConfig, SbasMode};

pub use geodetic::{Geodetic, GeographicCoordinate};

pub use gps_data::GpsData;
pub use gps_message::{GpsMessage, GpsYawHeadingMessage};
pub use gps_parser::{GpsParser, GpsParserEvent};
pub use gps_solution_data::{GpsSolutionData, GpsSolutionDataAbridged};

pub use nmea_gga::NmeaGga;
pub use nmea_gsa::NmeaGsa;
pub use nmea_gsv::NmeaGsv;
pub use nmea_parser::NmeaRecordType;
pub use nmea_rmc::NmeaRmc;

pub use ubx_nav_pvt_data::NavPvtData;
#[allow(unused)]
pub use ubx_parser::UbxMessage;

#[allow(unused)]
pub(crate) use ubx_nav_pvt_data::make_realistic_nav_pvt_payload;
