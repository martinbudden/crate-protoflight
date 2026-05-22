#![doc = include_str!("README.md")]

mod autopilot_task;
mod blackbox_task;
mod gyro_pid_task;
mod motor_mixer_task;
mod msp_task;
mod osd_task;
pub mod radio_task;

#[cfg(feature = "blackbox")]
pub(crate) use blackbox_task::BLACKBOX_CTX;
#[cfg(feature = "blackbox")]
pub use blackbox_task::{BlackboxContext, blackbox_task};

pub(crate) use gyro_pid_task::GYRO_CTX;
pub use gyro_pid_task::{GyroPidContext, gyro_pid_task};

pub(crate) use motor_mixer_task::MOTOR_MIXER_CTX;
pub use motor_mixer_task::{MotorMixerContext, motor_mixer_task};

#[cfg(feature = "osd")]
pub(crate) use osd_task::OSD_CTX;
#[cfg(feature = "osd")]
pub use osd_task::{OsdContext, osd_task};

pub(crate) use radio_task::RADIO_CTX;
pub use radio_task::{RadioContext, radio_task};

pub(crate) use msp_task::MSP_CTX;
pub use msp_task::{MspContext, msp_task};

pub(crate) use autopilot_task::AUTOPILOT_CTX;
pub use autopilot_task::{AutopilotContext, autopilot_task};

