#![cfg(feature = "msp")]
#![doc = include_str!("README.md")]

mod msp;
mod msp_commands;
mod msp_stream;

pub use msp::{Msp, MspSensorData};
#[allow(unused)]
pub use msp_stream::MspStream;
