#![cfg(feature = "msp")]
#![doc = include_str!("README.md")]

mod crc_dvb_s2;

mod msp;
mod msp_commands;
mod msp_parser;
mod msp_port;
mod msp_stream;

pub use msp::{Msp, MspSensorData};
pub use msp_parser::MspParser;
pub use msp_port::MspPort;
pub use msp_stream::MspStream;
