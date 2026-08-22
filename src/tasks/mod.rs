//#![allow(clippy::doc_markdown)]
#![doc = include_str!("README.md")]

// Macros must be brought into scope before the modules that use them.

#[macro_use]
mod non_volatile_storage_macros;
#[macro_use]
mod global_debug;

mod autopilot;
mod barometer;
mod battery;
mod blackbox_encoder;
mod blackbox_writer;
mod gps;
mod gyro_pid;
mod init;
mod magnetometer;
mod motor_mixer;
mod msp;
mod non_volatile_storage;
mod optical_flow;
mod osd;
mod rangefinder;
mod rx;

pub use init::init;

#[allow(unused)]
#[cfg(feature = "debug")]
pub use global_debug::{DebugMode, GLOBAL_DEBUG, GlobalDebug};
