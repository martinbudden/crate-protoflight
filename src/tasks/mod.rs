#![doc = include_str!("README.md")]

mod autopilot_task;
mod barometer_task;
mod blackbox_task;
mod gyro_pid_task;
mod motor_mixer_task;
mod msp_task;
mod osd_task;
pub mod radio_task;

pub(crate) use gyro_pid_task::GYRO_CTX;
pub use gyro_pid_task::{GyroPidContext, gyro_pid_task};

pub(crate) use motor_mixer_task::MOTOR_MIXER_CTX;
pub use motor_mixer_task::{MotorMixerContext, motor_mixer_task};

pub(crate) use msp_task::MSP_CTX;
pub use msp_task::{MspContext, msp_task};

pub(crate) use radio_task::RADIO_CTX;
pub use radio_task::{RadioContext, radio_task};

#[cfg(feature = "autopilot")]
pub(crate) use autopilot_task::AUTOPILOT_CTX;
#[cfg(feature = "autopilot")]
pub use autopilot_task::{AutopilotContext, autopilot_task};

#[cfg(feature = "barometer")]
pub(crate) use barometer_task::BAROMETER_CTX;
#[cfg(feature = "barometer")]
pub use barometer_task::{BarometerContext, barometer_task};

#[cfg(feature = "blackbox")]
pub(crate) use blackbox_task::BLACKBOX_CTX;
#[cfg(feature = "blackbox")]
pub use blackbox_task::{BlackboxContext, blackbox_task};

#[cfg(feature = "osd")]
pub(crate) use osd_task::OSD_CTX;
#[cfg(feature = "osd")]
pub use osd_task::{OsdContext, osd_task};
