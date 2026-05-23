#![allow(unused)]
#![doc = include_str!("README.md")]

mod data;

pub use data::SENSOR_DATA;
pub use data::{SENSOR_DATA_PUB_SUB_CHANNEL, SensorDataItem, SensorDataPublisher, SensorDataSubscriber};
