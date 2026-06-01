#![allow(unused)]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
use vqm::Vector3df32;

use crate::config::ConfigSubscriber;
use crate::gps::GpsSolutionData;
use crate::sensors::{BarometerData, GpsData, GpsPosition, GpsYawHeadingData, RangefinderData};

/// The global configuration is a global static protected by a mutex, since it is used by several tasks.
/// A `CriticalSectionRawMutex` is used since we need to be safe across multiple executors and interrupts.
static SENSOR_DATA: Mutex<CriticalSectionRawMutex, SensorData> = Mutex::new(SensorData::new());

const MAX_SENSOR_DATA_SUBSCRIBER_COUNT: usize = 10;
const SENSOR_DATA_PUBLISHER_COUNT: usize = 2;
const SENSOR_DATA_CAPACITY: usize = 8;

/// `PubSubChannel` for handling `SensorData` updates.
static SENSOR_DATA_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    SensorDataItem,
    SENSOR_DATA_CAPACITY,
    MAX_SENSOR_DATA_SUBSCRIBER_COUNT,
    SENSOR_DATA_PUBLISHER_COUNT,
> = PubSubChannel::new();

pub type SensorDataPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    SensorDataItem,
    SENSOR_DATA_CAPACITY,
    MAX_SENSOR_DATA_SUBSCRIBER_COUNT,
    SENSOR_DATA_PUBLISHER_COUNT,
>;

pub fn sensor_data_publisher<'a>() -> SensorDataPublisher<'a> {
    SENSOR_DATA_PUB_SUB_CHANNEL.publisher().expect("sensor_data_publisher failed")
}

pub type SensorDataSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    SensorDataItem,
    SENSOR_DATA_CAPACITY,
    MAX_SENSOR_DATA_SUBSCRIBER_COUNT,
    SENSOR_DATA_PUBLISHER_COUNT,
>;

pub fn sensor_data_subscriber<'a>() -> SensorDataSubscriber<'a> {
    SENSOR_DATA_PUB_SUB_CHANNEL.subscriber().expect("sensor_data_subscriber failed")
}

/// The only subscriber is the `gyro_pid_task`.
const MAX_FAST_SENSOR_DATA_SUBSCRIBER_COUNT: usize = 1;
const FAST_SENSOR_DATA_PUBLISHER_COUNT: usize = 2;
const FAST_SENSOR_DATA_CAPACITY: usize = 8;

/// High speed `PubSubChannel` for handling `SensorData` updates in the  `gyro_pid` task.
static FAST_SENSOR_DATA_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    FastSensorDataItem,
    FAST_SENSOR_DATA_CAPACITY,
    MAX_FAST_SENSOR_DATA_SUBSCRIBER_COUNT,
    FAST_SENSOR_DATA_PUBLISHER_COUNT,
> = PubSubChannel::new();

pub type FastSensorDataPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    FastSensorDataItem,
    FAST_SENSOR_DATA_CAPACITY,
    MAX_FAST_SENSOR_DATA_SUBSCRIBER_COUNT,
    FAST_SENSOR_DATA_PUBLISHER_COUNT,
>;

pub fn fast_sensor_data_publisher<'a>() -> FastSensorDataPublisher<'a> {
    FAST_SENSOR_DATA_PUB_SUB_CHANNEL.publisher().expect("fast_sensor_data_publisher failed")
}

pub type FastSensorDataSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    FastSensorDataItem,
    FAST_SENSOR_DATA_CAPACITY,
    MAX_FAST_SENSOR_DATA_SUBSCRIBER_COUNT,
    FAST_SENSOR_DATA_PUBLISHER_COUNT,
>;

pub fn fast_sensor_data_subscriber<'a>() -> FastSensorDataSubscriber<'a> {
    FAST_SENSOR_DATA_PUB_SUB_CHANNEL.subscriber().expect("fast_sensor_data_subscriber failed")
}

/// Macro to generate the `SensorData` struct.<br>
/// Creates a new function that calls the new function of all the member configs.<br>
/// Also generates data-carrying enums for the `sensor_data` `PubSubChannel`.<br>
/// All sensor data must define a const `new` function.<br>
macro_rules! define_sensor_data {
    (
        general: [ $( $(#[$g_meta:meta])* ($g_enum:ident, $g_field:ident, $g_type:ty)),* $(,)?],
        fast: [ $( $(#[$p_meta:meta])* ($p_enum:ident, $p_field:ident, $p_type:ty)),* $(,)?]
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct SensorData {
            $(
                $(#[$g_meta])*
                pub $g_field: $g_type,
            )*
            $(
                $(#[$p_meta])*
                pub $p_field: $p_type,
            )*
        }

        impl SensorData {
            pub const fn new() -> Self {
                Self {
                    $(
                        $(#[$g_meta])*
                        $g_field: <$g_type>::new(),
                    )*
                    $(
                        $(#[$p_meta])*
                        $p_field: <$p_type>::new(),
                    )*
                 }
            }
        }

        impl Default for SensorData {
            fn default() -> Self {
                Self::new()
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        #[non_exhaustive]
        pub enum SensorDataItem {
            $(
                $(#[$g_meta])*
                $g_enum($g_type)
            ),*
        }
        #[derive(Clone, Copy, Debug, PartialEq)]
        #[non_exhaustive]
        pub enum FastSensorDataItem {
            $(
                $(#[$p_meta])*
                $p_enum($p_type)
            ),*
        }
    };
}

define_sensor_data!(
    // (enum, field, struct)
    general: [
        (Barometer, barometer, BarometerData),
        (Rangefinder, rangefinder, RangefinderData),
        (Gps, gps, GpsData),
        (GpsPosition, gps_position, GpsPosition),
        (GpsSolution, gps_solution_data, GpsSolutionData),
    ],
    fast: [
        (GpsYawHeading, gps_yaw_heading_radians, GpsYawHeadingData),
    ]

);

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<SensorData>();
    }
    #[test]
    fn global_config_new() {
        let _config = SensorData::new();
    }
}
