#![doc = include_str!("README.md")]

mod global_config;
mod imu_config;
mod profiles;
mod system_config;

#[allow(unused)] // used by MSP
pub use global_config::{config_publisher, fast_config_publisher};

pub use global_config::GLOBAL_CONFIG;
pub use global_config::{ConfigItem, ConfigPublisher, ConfigSubscriber, config_subscriber};
pub use global_config::{FastConfigItem, FastConfigPublisher, FastConfigSubscriber, fast_config_subscriber};
pub use imu_config::ImuConfig;
pub use system_config::SystemConfig;
