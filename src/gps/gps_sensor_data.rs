#![cfg(feature = "gps")]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};

use crate::gps::{GpsData, GpsPosition, GpsSolutionData, GpsYawHeadingData};

const MAX_GPS_DATA_SUBSCRIBER_COUNT: usize = 8;
const GPS_DATA_PUBLISHER_COUNT: usize = 1;
const GPS_DATA_CAPACITY: usize = 4;

/// `PubSubChannel` for handling `GpsData` updates.
static GPS_DATA_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    GpsDataItem,
    GPS_DATA_CAPACITY,
    MAX_GPS_DATA_SUBSCRIBER_COUNT,
    GPS_DATA_PUBLISHER_COUNT,
> = PubSubChannel::new();

pub type GpsDataPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    GpsDataItem,
    GPS_DATA_CAPACITY,
    MAX_GPS_DATA_SUBSCRIBER_COUNT,
    GPS_DATA_PUBLISHER_COUNT,
>;

pub fn gps_data_publisher<'a>() -> GpsDataPublisher<'a> {
    GPS_DATA_PUB_SUB_CHANNEL.publisher().expect("sensor_data_publisher failed")
}

pub type GpsDataSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    GpsDataItem,
    GPS_DATA_CAPACITY,
    MAX_GPS_DATA_SUBSCRIBER_COUNT,
    GPS_DATA_PUBLISHER_COUNT,
>;

pub fn gps_data_subscriber<'a>() -> GpsDataSubscriber<'a> {
    GPS_DATA_PUB_SUB_CHANNEL.subscriber().expect("sensor_data_subscriber failed")
}

/// The only subscriber is the `gyro_pid_task`.
const MAX_YAW_HEADING_SUBSCRIBER_COUNT: usize = 1;
const YAW_HEADING_PUBLISHER_COUNT: usize = 1;
const YAW_HEADING_CAPACITY: usize = 1;

/// High speed `PubSubChannel` for handling `GpsData` updates in the  `gyro_pid` task.
static YAW_HEADING_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    GpsYawHeadingData,
    YAW_HEADING_CAPACITY,
    MAX_YAW_HEADING_SUBSCRIBER_COUNT,
    YAW_HEADING_PUBLISHER_COUNT,
> = PubSubChannel::new();

pub type YawHeadingPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    GpsYawHeadingData,
    YAW_HEADING_CAPACITY,
    MAX_YAW_HEADING_SUBSCRIBER_COUNT,
    YAW_HEADING_PUBLISHER_COUNT,
>;

pub fn yaw_heading_publisher<'a>() -> YawHeadingPublisher<'a> {
    YAW_HEADING_PUB_SUB_CHANNEL.publisher().expect("yaw_heading_publisher failed")
}

pub type YawHeadingSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    GpsYawHeadingData,
    YAW_HEADING_CAPACITY,
    MAX_YAW_HEADING_SUBSCRIBER_COUNT,
    YAW_HEADING_PUBLISHER_COUNT,
>;

pub fn yaw_heading_subscriber<'a>() -> YawHeadingSubscriber<'a> {
    YAW_HEADING_PUB_SUB_CHANNEL.subscriber().expect("yaw_heading_subscriber failed")
}

/*
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
        (Gps, gps, GpsData),
        (GpsPosition, gps_position, GpsPosition),
        (GpsSolution, gps_solution_data, GpsSolutionData),
    ],
    fast: [
//        (GpsYawHeading, gps_yaw_heading_radians, GpsYawHeadingData),
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
*/

/*#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensorData {
    pub gps: GpsData,
    pub gps_position: GpsPosition,
    pub gps_solution_data: GpsSolutionData,
}
impl SensorData {
    pub const fn new() -> Self {
        Self { gps: <GpsData>::new(), gps_position: <GpsPosition>::new(), gps_solution_data: <GpsSolutionData>::new() }
    }
}
impl Default for SensorData {
    fn default() -> Self {
        Self::new()
    }
}*/

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum GpsDataItem {
    Gps(GpsData),
    GpsPosition(GpsPosition),
    GpsSolution(GpsSolutionData),
}
