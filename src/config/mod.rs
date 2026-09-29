#![doc = include_str!("README.md")]

mod failsafe;
mod global_config;
mod imu_config;
mod profiles;
mod rc_controls_config;
mod rx_config;
mod sensor_flags;
mod system_config;

#[allow(unused)] // used by MSP
pub use global_config::{config_publisher, fast_config_publisher};

pub use failsafe::{FailsafeConfig, FailsafeProcedure, FailsafeSwitchMode};
pub use global_config::GLOBAL_CONFIG;
pub use global_config::{ConfigItem, ConfigPublisher, ConfigSubscriber, config_subscriber};
pub use global_config::{FastConfigItem, FastConfigPublisher, FastConfigSubscriber, fast_config_subscriber};
pub use imu_config::ImuConfig;
pub use rc_controls_config::RcControlsConfig;
pub use rx_config::RxConfig;
pub use sensor_flags::SensorFlags;
pub use system_config::SystemConfig;
