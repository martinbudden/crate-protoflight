//#![allow(clippy::doc_markdown)]
#![doc = include_str!("README.md")]

mod autopilot_task;
mod barometer_task;
mod battery_task;
mod blackbox_task;
mod flight_control_task;
mod gps_task;
mod gyro_pid_task;
mod imu_task;
pub mod init;
mod motor_mixer_task;
mod msp_task;
mod non_volatile_storage;
mod osd_task;
mod rangefinder_task;
