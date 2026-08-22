#![doc = include_str!("README.md")]
#![allow(unused)]

pub mod nmea;
pub mod ubx;

mod gps_config;
mod gps_rescue_config;

mod geodetic;
mod gps_data;
mod gps_message;
mod gps_parser;
mod gps_status_data;

pub use gps_config::{GpsConfig, GpsOffOn, GpsProvider, SbasMode};
pub use gps_rescue_config::{GpsRescueAltitudeMode, GpsRescueConfig, GpsRescueSanityChecks};

pub use geodetic::{Geodetic, GeographicCoordinate};

pub use gps_data::{GpsData, GpsDataAbridged};
pub use gps_message::{GpsMessage, GpsYawHeadingMessage};
pub use gps_parser::{GpsParser, GpsParserEvent};
pub use gps_status_data::GpsStatusData;

pub use nmea::{NmeaGga, NmeaGsa, NmeaGsv, NmeaRecordType, NmeaRmc};

pub use ubx::{
    UbxAckAck, UbxAckId, UbxAckNak, UbxCfgId, UbxCfgMsgPoll, UbxCfgMsgSet, UbxCfgNav5, UbxCfgPms, UbxCfgRate,
    UbxClassId, UbxMessage, UbxMonId, UbxMonVer, UbxNavDop, UbxNavId, UbxNavPvt, UbxNmeaId, UbxParser,
};

pub(crate) use ubx::make_realistic_nav_pvt_payload;
