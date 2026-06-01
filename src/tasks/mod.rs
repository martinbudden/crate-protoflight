//#![allow(clippy::doc_markdown)]
#![doc = include_str!("README.md")]

mod autopilot_task;
mod barometer_task;
mod blackbox_task;
mod gps_task;
mod gyro_pid_task;
pub(crate) mod init;
mod motor_mixer_task;
mod msp_task;
mod non_volatile_storage;
mod osd_task;
mod radio_task;
mod rangefinder_task;

pub(crate) use gyro_pid_task::{GyroPidContext, gyro_pid_task};

pub(crate) use motor_mixer_task::{MotorMixerContext, motor_mixer_task};

pub(crate) use radio_task::{RadioContext, radio_task};

#[cfg(feature = "autopilot")]
pub(crate) use autopilot_task::{AutopilotContext, autopilot_task};

#[cfg(feature = "barometer")]
pub(crate) use barometer_task::{BarometerContext, barometer_task};

#[cfg(feature = "blackbox")]
pub(crate) use blackbox_task::{BlackboxContext, blackbox_task};

#[cfg(feature = "gps")]
pub(crate) use gps_task::{GpsContext, gps_task};

#[cfg(feature = "msp")]
pub(crate) use msp_task::{MSP_READ_BUF_SIZE, MSP_WRITE_BUF_SIZE, MspContext, msp_task};

#[cfg(feature = "osd")]
pub(crate) use osd_task::{OsdContext, osd_task};

#[cfg(feature = "rangefinder")]
pub(crate) use rangefinder_task::{RangefinderContext, rangefinder_task};
