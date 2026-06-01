//#![allow(clippy::doc_markdown)]
#![doc = include_str!("README.md")]

mod autopilot_task;
mod barometer_task;
mod blackbox_task;
mod dispatch;
mod gps_task;
mod gyro_pid_task;
pub(crate) mod init;
mod motor_mixer_task;
mod msp_task;
mod non_volatile_storage;
mod osd_task;
mod radio_task;
mod rangefinder_task;

pub(crate) use gyro_pid_task::{GYRO_CTX, GyroPidContext, gyro_pid_task};

pub(crate) use motor_mixer_task::{MOTOR_MIXER_CTX, MotorMixerContext, motor_mixer_task};

pub(crate) use radio_task::{RADIO_CTX, RadioContext, radio_task};

#[cfg(feature = "autopilot")]
pub(crate) use autopilot_task::{AUTOPILOT_CTX, AutopilotContext, autopilot_task};

#[cfg(feature = "barometer")]
pub(crate) use barometer_task::{BAROMETER_CTX, BarometerContext, barometer_task};

#[cfg(feature = "blackbox")]
pub(crate) use blackbox_task::{BLACKBOX_CTX, BlackboxContext, blackbox_task};

#[cfg(feature = "gps")]
pub(crate) use gps_task::{GPS_CTX, GpsContext, gps_task};

#[cfg(feature = "msp")]
pub(crate) use msp_task::{MSP_CTX, MSP_READ_BUF_SIZE, MSP_WRITE_BUF_SIZE, MspContext, msp_task};

#[cfg(feature = "osd")]
pub(crate) use osd_task::{OSD_CTX, OsdContext, osd_task};

#[cfg(feature = "rangefinder")]
pub(crate) use rangefinder_task::{RANGEFINDER_CTX, RangefinderContext, rangefinder_task};
