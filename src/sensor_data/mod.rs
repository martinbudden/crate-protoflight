#![doc = include_str!("README.md")]

mod global_sensor_data;

pub use global_sensor_data::{
    FastSensorDataItem, FastSensorDataPublisher, FastSensorDataSubscriber, fast_sensor_data_publisher,
    fast_sensor_data_subscriber,
};
pub use global_sensor_data::{
    SensorDataItem, SensorDataPublisher, SensorDataSubscriber, sensor_data_publisher, sensor_data_subscriber,
};
