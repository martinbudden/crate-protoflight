//#![allow(clippy::doc_markdown)]
#![doc = include_str!("README.md")]

#[macro_use]
pub mod global_debug;

mod autopilot;
mod barometer;
mod battery;
mod blackbox;
mod blackbox_writer;
mod gps;
mod gyro_pid;
mod imu;
pub mod init;
mod magnetometer;
mod motor_mixer;
mod msp;
mod non_volatile_storage;
mod optical_flow;
mod osd;
mod rangefinder;
mod rx;

#[allow(unused)]
#[cfg(feature = "debug")]
pub use global_debug::{DebugMode, GLOBAL_DEBUG};
