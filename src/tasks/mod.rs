//#![allow(clippy::doc_markdown)]
#![doc = include_str!("README.md")]

#[macro_use]
pub mod global_debug;

mod autopilot_task;
mod barometer_task;
mod battery_task;
mod blackbox_task;
mod blackbox_writer_task;
mod gps_task;
mod gyro_pid_task;
mod imu_task;
pub mod init;
mod init_rp;
mod motor_mixer_task;
mod msp_task;
mod non_volatile_storage;
mod optical_flow_task;
mod osd_task;
mod rangefinder_task;
mod rx_task;

#[allow(unused)]
#[cfg(feature = "debug")]
pub use global_debug::{DebugMode, GLOBAL_DEBUG};
