#![doc = include_str!("README.md")]

mod global_config;
mod profiles;

pub use global_config::{CONFIG_PUB_SUB_CHANNEL, ConfigItem, ConfigPublisher, ConfigSubscriber};
pub use global_config::{GLOBAL_CONFIG};
pub use global_config::{GYRO_PID_PUB_SUB_CHANNEL, GyroPidItem, GyroPidPublisher, GyroPidSubscriber};
