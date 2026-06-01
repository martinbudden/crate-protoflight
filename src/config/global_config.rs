use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};

use motor_mixers::{MixerConfig, MotorConfig, MotorDeviceConfig};
use radio_controllers::{FailsafeConfig, RatesConfig, RcControlsConfig, RcModes, RxConfig};

use crate::autopilot::{AutopilotConfig, PositionHoldConfig};
use crate::config::{
    ImuConfig,
    profiles::{PidProfile, RatesProfile, SchemaVersion},
};
use crate::flight::{
    AntiGravityConfig, ArmingConfig, CrashFlipConfig, CrashRecoveryConfig, DMaxConfig, FeatureConfig,
    FlightControllerFiltersConfig, GyroConfig, ImuFilterBankConfig, PidConfig, TpaConfig, YawSpinRecoveryConfig,
};
use crate::sensors::BatteryConfig;

#[cfg(feature = "gps")]
use crate::gps::{GpsConfig, GpsRescueConfig};

#[cfg(feature = "osd")]
use crate::osd::{OsdConfig, OsdElementsConfig, OsdStatsConfig};

#[cfg(feature = "vtx")]
use crate::vtx::{Vtx, VtxConfig};

#[cfg(feature = "blackbox")]
use blackbox_logger::BlackboxConfig;

/// The global configuration is a global static protected by a mutex, since it is used by several tasks.
/// A `CriticalSectionRawMutex` is used since we need to be safe across multiple executors and interrupts.
pub static GLOBAL_CONFIG: Mutex<CriticalSectionRawMutex, GlobalConfig> = Mutex::new(GlobalConfig::new());

const MAX_CONFIG_SUBSCRIBER_COUNT: usize = 10;
const CONFIG_PUBLISHER_COUNT: usize = 2;
const CONFIG_CAPACITY: usize = 8;

/// `PubSubChannel` for handling `GlobalConfig` updates.
static CONFIG_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    ConfigItem,
    CONFIG_CAPACITY,
    MAX_CONFIG_SUBSCRIBER_COUNT,
    CONFIG_PUBLISHER_COUNT,
> = PubSubChannel::new();

pub type ConfigPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    ConfigItem,
    CONFIG_CAPACITY,
    MAX_CONFIG_SUBSCRIBER_COUNT,
    CONFIG_PUBLISHER_COUNT,
>;

#[allow(unused)]
pub fn config_publisher<'a>() -> ConfigPublisher<'a> {
    CONFIG_PUB_SUB_CHANNEL.publisher().expect("config_publisher failed")
}

pub type ConfigSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    ConfigItem,
    CONFIG_CAPACITY,
    MAX_CONFIG_SUBSCRIBER_COUNT,
    CONFIG_PUBLISHER_COUNT,
>;

pub fn config_subscriber<'a>() -> ConfigSubscriber<'a> {
    CONFIG_PUB_SUB_CHANNEL.subscriber().expect("config_subscriber failed")
}

/// The only subscriber is the `gyro_pid_task`.
const FAST_CONFIG_SUBSCRIBER_COUNT: usize = 1;
const FAST_CONFIG_PUBLISHER_COUNT: usize = 2;
const FAST_CONFIG_CAPACITY: usize = 4;

/// High speed `PubSubChannel` for handling `GlobalConfig` updates in the  `gyro_pid` task.
static FAST_CONFIG_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    FastConfigItem,
    FAST_CONFIG_CAPACITY,
    FAST_CONFIG_SUBSCRIBER_COUNT,
    FAST_CONFIG_PUBLISHER_COUNT,
> = PubSubChannel::new();

pub type FastConfigPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    FastConfigItem,
    FAST_CONFIG_CAPACITY,
    FAST_CONFIG_SUBSCRIBER_COUNT,
    FAST_CONFIG_PUBLISHER_COUNT,
>;

#[allow(unused)]
pub fn fast_config_publisher<'a>() -> FastConfigPublisher<'a> {
    FAST_CONFIG_PUB_SUB_CHANNEL.publisher().expect("fast_config_publisher failed")
}

pub type FastConfigSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    FastConfigItem,
    FAST_CONFIG_CAPACITY,
    FAST_CONFIG_SUBSCRIBER_COUNT,
    FAST_CONFIG_PUBLISHER_COUNT,
>;

pub fn fast_config_subscriber<'a>() -> FastConfigSubscriber<'a> {
    FAST_CONFIG_PUB_SUB_CHANNEL.subscriber().expect("fast_config_subscriber failed")
}

/// Macro to generate the `GlobalConfig` struct.<br>
/// Creates a new function that calls the new function of all the member configs.<br>
/// Also generates data-carrying enums for the `config` and `gyro_pid` `PubSubChannel`s.<br>
/// All member configs must define a const `new` function.<br>
macro_rules! define_configs {
    (
        general: [ $( $(#[$g_meta:meta])* ($g_enum:ident, $g_field:ident, $g_type:ty)),* $(,)?],
        fast: [ $( $(#[$p_meta:meta])* ($p_enum:ident, $p_field:ident, $p_type:ty)),* $(,)?]
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct GlobalConfig {
            $(
                $(#[$g_meta])*
                pub $g_field: $g_type,
            )*
            $(
                $(#[$p_meta])*
                pub $p_field: $p_type,
            )*
        }

        impl GlobalConfig {
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

        impl Default for GlobalConfig {
            fn default() -> Self {
                Self::new()
            }
        }

        #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
        pub enum ConfigItem {
            $(
                $(#[$g_meta])*
                $g_enum($g_type)
            ),*
        }

        #[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
        pub enum FastConfigItem {
            $(
                $(#[$p_meta])*
                $p_enum($p_type)
            ),*
        }
    };
}

define_configs!(
    // (enum, field, struct)
    general: [
        (Schema, schema_version, SchemaVersion),
        (PidProfile, pid_profile, PidProfile),
        (RatesProfile, rates_profile, RatesProfile),
        (Rates, rates, RatesConfig),
        (Failsafe, failsafe, FailsafeConfig),
        (FlightControlFilters, flight_control_filters, FlightControllerFiltersConfig),
        (ImuFilters, imu_filter_bank, ImuFilterBankConfig),
        (Gyro, gyro, GyroConfig),
        (Mixer, mixer, MixerConfig),
        (Motor, motor, MotorConfig),
        (MotorDevice, motor_device, MotorDeviceConfig),
        (Tpa, tpa, TpaConfig),
        (YawSpinRecovery, yaw_spin_recovery, YawSpinRecoveryConfig),
        (CrashFlip, crash_flip, CrashFlipConfig),
        (CrashRecovery, crash_recovery, CrashRecoveryConfig),
        (AntiGravity, anti_gravity, AntiGravityConfig),
        (DMax, dmax, DMaxConfig),
        (Rx, rx, RxConfig),
        (RcModes, rc_modes, RcModes),
        (RcControls, rc_controls, RcControlsConfig),
        (Arming, arming, ArmingConfig),
        (Features, features, FeatureConfig),
        (Autopilot, autopilot, AutopilotConfig),
        (PositionHold, position_hold, PositionHoldConfig),
        (Battery, battery, BatteryConfig),
        (Imu, imu, ImuConfig),

        #[cfg(feature = "blackbox")]
        (Blackbox, blackbox, BlackboxConfig),

        #[cfg(feature = "vtx")]
        (Vtx, vtx, VtxConfig),

        #[cfg(feature = "gps")]
        (Gps, gps, GpsConfig),
        #[cfg(feature = "gps")]
        (GpsRescue, gps_rescue, GpsRescueConfig),

        #[cfg(feature = "osd")]
        (Osd, osd, OsdConfig),
        #[cfg(feature = "osd")]
        (OsdElements, osd_elements, OsdElementsConfig),
        #[cfg(feature = "osd")]
        (OsdStats, osd_stats, OsdStatsConfig),
    ],
    // GyroPid configs are defined separately so they can have their own PubSub channel.
    fast: [
        (RollRate, pid_roll_rate, PidConfig),
        (PitchRate, pid_pitch_rate, PidConfig),
        (YawRate, pid_yaw_rate, PidConfig),
        (RollAngle, pid_roll_angle, PidConfig),
        (PitchAngle, pid_pitch_angle, PidConfig),
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use {
        sequential_storage::map::PostcardValue,
        serde::{Deserialize, Serialize},
    };

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_full_no_default<T: Sized + Send + Sync + Unpin + Copy + Clone + PartialEq>() {}
    fn _is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<GlobalConfig>();
        is_full_no_default::<ConfigItem>();
        is_full_no_default::<FastConfigItem>();
    }
    #[test]
    fn global_config_new() {
        let _config = GlobalConfig::new();
    }
}
