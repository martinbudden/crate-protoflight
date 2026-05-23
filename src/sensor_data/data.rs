#![allow(unused)]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};

use crate::sensors::BarometerData;

/// The global configuration is a global static protected by a mutex, since it is used by several tasks.
/// A `CriticalSectionRawMutex` is used since we need to be safe across multiple executors and interrupts.
pub static SENSOR_DATA: Mutex<CriticalSectionRawMutex, SensorData> = Mutex::new(SensorData::new());

const MAX_SENSOR_DATA_SUBSCRIBER_COUNT: usize = 10;
const SENSOR_DATA_PUBLISHER_COUNT: usize = 2;
const SENSOR_DATA_CAPACITY: usize = 8;

/// `PubSubChannel` for handling `SensorData` updates.
pub static SENSOR_DATA_PUB_SUB_CHANNEL: PubSubChannel<
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

pub type SensorDataSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    SensorDataItem,
    SENSOR_DATA_CAPACITY,
    MAX_SENSOR_DATA_SUBSCRIBER_COUNT,
    SENSOR_DATA_PUBLISHER_COUNT,
>;

/// Macro to generate the `SensorData` struct.<br>
/// Creates a new function that calls the new function of all the member configs.<br>
/// Also generates data-carrying enums for the `sensor_data` `PubSubChannel`.<br>
/// All sensor data must define a const `new` function.<br>
macro_rules! define_sensor_data {
    (
        general: [ $( $(#[$g_meta:meta])* ($g_enum:ident, $g_field:ident, $g_type:ty)),* $(,)?],
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct SensorData {
            $(
                $(#[$g_meta])*
                pub $g_field: $g_type,
            )*
        }

        impl SensorData {
            pub const fn new() -> Self {
                Self {
                    $(
                        $(#[$g_meta])*
                        $g_field: <$g_type>::new(),
                    )*
                 }
            }
        }

        impl Default for SensorData {
            fn default() -> Self {
                Self::new()
            }
        }
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum SensorDataItem {
            $(
                $(#[$g_meta])*
                $g_enum($g_type)
            ),*
        }
    };
}

define_sensor_data!(
    // (enum, field, struct)
    general: [
        (Barometer, barometer, BarometerData),
    ],
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
