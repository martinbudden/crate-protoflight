#![doc = include_str!("README.md")]
#![allow(unused)]

mod gps_config;
mod gps_rescue_config;

mod geodetic;
mod gps_data;
mod gps_message;
mod gps_parser;
mod gps_status_data;

mod nmea_gga;
mod nmea_gsa;
mod nmea_gsv;
mod nmea_parser;
mod nmea_rmc;

mod ubx_ack;
mod ubx_ack_ack;
mod ubx_ack_nak;

mod ubx_cfg;
mod ubx_cfg_msg;
mod ubx_cfg_nav5;
mod ubx_cfg_pms;
mod ubx_cfg_rate;

mod ubx_mon;
mod ubx_mon_ver;

mod ubx_nav;
mod ubx_nav_dop;
mod ubx_nav_posllh;
mod ubx_nav_pvt;
mod ubx_nav_status;
mod ubx_nav_velned;
mod ubx_nmea;
mod ubx_parser;

pub use gps_config::{GpsConfig, GpsOffOn, GpsProvider, SbasMode};
pub use gps_rescue_config::{GpsRescueAltitudeMode, GpsRescueConfig, GpsRescueSanityChecks};

pub use geodetic::{Geodetic, GeographicCoordinate};

pub use gps_data::{GpsData, GpsDataAbridged};
pub use gps_message::{GpsMessage, GpsYawHeadingMessage};
pub use gps_parser::{GpsParser, GpsParserEvent};
pub use gps_status_data::GpsStatusData;

pub use nmea_gga::NmeaGga;
pub use nmea_gsa::NmeaGsa;
pub use nmea_gsv::NmeaGsv;
pub use nmea_parser::NmeaRecordType;
pub use nmea_rmc::NmeaRmc;

pub use ubx_nmea::UbxNmeaId;
pub use ubx_parser::{UbxClassId, UbxMessage};

pub use ubx_ack::UbxAckId;
pub use ubx_ack_ack::UbxAckAck;
pub use ubx_ack_nak::UbxAckNak;

pub use ubx_cfg::UbxCfgId;
pub use ubx_cfg_msg::{UbxCfgMsgPoll, UbxCfgMsgSet};
pub use ubx_cfg_nav5::UbxCfgNav5;
pub use ubx_cfg_pms::UbxCfgPms;
pub use ubx_cfg_rate::UbxCfgRate;

pub use ubx_mon::UbxMonId;
pub use ubx_mon_ver::UbxMonVer;

pub use ubx_nav::UbxNavId;
pub use ubx_nav_dop::UbxNavDop;
pub use ubx_nav_pvt::UbxNavPvt;

#[allow(unused)]
pub(crate) use ubx_nav_pvt::make_realistic_nav_pvt_payload;
