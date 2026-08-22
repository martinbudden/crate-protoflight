#![doc = include_str!("README.md")]
#![allow(unused)]

pub mod nmea;
pub mod ubx;

mod gps_config;
mod gps_rescue_config;

mod geodetic;
mod gps_message;
mod gps_parser;
mod gps_solution;
mod gps_status;

pub use gps_config::{GpsConfig, GpsOffOn, GpsProvider, SbasMode};
pub use gps_rescue_config::{GpsRescueAltitudeMode, GpsRescueConfig, GpsRescueSanityChecks};

pub use geodetic::{Geodetic, GeographicCoordinate};

pub use gps_message::{GpsMessage, GpsYawHeadingMessage};
pub use gps_parser::{GpsParser, GpsParserEvent};
pub use gps_solution::{GpsSolution, GpsSolutionAbridged};
pub use gps_status::GpsStatus;

pub use nmea::{NmeaGga, NmeaGsa, NmeaGsv, NmeaRecordType, NmeaRmc};

pub use ubx::{
    UbxAckAck, UbxAckNak, UbxCfgMsgPoll, UbxCfgMsgSet, UbxCfgNav5, UbxCfgPms, UbxCfgRate, UbxMessage, UbxMonId,
    UbxMonVer, UbxNavDop, UbxNavId, UbxNavPvt, UbxParser,
};

pub(crate) use ubx::make_realistic_nav_pvt_payload;
