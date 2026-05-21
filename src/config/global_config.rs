use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};

use blackbox_logger::BlackboxConfig;
use motor_mixers::{MixerConfig, MotorConfig, MotorDeviceConfig};
use radio_controllers::{FailsafeConfig, RatesConfig, RcControlsConfig, RcModes, RxConfig};

use crate::autopilot::{AutopilotConfig, PositionHoldConfig};
use crate::config::profiles::{PidProfile, RatesProfile, SchemaVersion};
use crate::flight::{
    AntiGravityConfig, ArmingConfig, CrashFlipConfig, CrashRecoveryConfig, DMaxConfig, FeatureConfig,
    FlightControllerFiltersConfig, GyroConfig, ImuFilterBankConfig, PidConfig, TpaConfig, YawSpinRecoveryConfig,
};
#[cfg(feature = "gps")]
use crate::gps::GpsConfig;
#[cfg(feature = "osd")]
use crate::osd::{OsdConfig, OsdElementsConfig, OsdStatsConfig};
use crate::sensors::BatteryConfig;
#[cfg(feature = "vtx")]
use crate::vtx::{Vtx, VtxConfig};

/// The global configuration is a global static protected by a mutex, since it is used by several tasks.
/// A `CriticalSectionRawMutex` is used since we need to be safe across multiple executors and interrupts.
pub static GLOBAL_CONFIG: Mutex<CriticalSectionRawMutex, GlobalConfig> = Mutex::new(GlobalConfig::new());

const MAX_CONFIG_SUBSCRIBER_COUNT: usize = 10;
const CONFIG_PUBLISHER_COUNT: usize = 2;
/// `PubSubChannel` for handling `GlobalConfig` updates.
pub static CONFIG_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    ConfigItem,
    8,
    MAX_CONFIG_SUBSCRIBER_COUNT,
    CONFIG_PUBLISHER_COUNT,
> = PubSubChannel::new();

pub type ConfigPublisher<'a> =
    Publisher<'a, CriticalSectionRawMutex, ConfigItem, 8, MAX_CONFIG_SUBSCRIBER_COUNT, CONFIG_PUBLISHER_COUNT>;
pub type ConfigSubscriber<'a> =
    Subscriber<'a, CriticalSectionRawMutex, ConfigItem, 8, MAX_CONFIG_SUBSCRIBER_COUNT, CONFIG_PUBLISHER_COUNT>;

const MAX_GYRO_PID_SUBSCRIBER_COUNT: usize = 4;
const GYRO_PID_PUBLISHER_COUNT: usize = 2;
/// High speed `PubSubChannel` for handling `GlobalConfig` updates in the  `gyro_pid` task.
pub static GYRO_PID_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    GyroPidItem,
    4,
    MAX_GYRO_PID_SUBSCRIBER_COUNT,
    GYRO_PID_PUBLISHER_COUNT,
> = PubSubChannel::new();

pub type GyroPidPublisher<'a> =
    Publisher<'a, CriticalSectionRawMutex, GyroPidItem, 4, MAX_GYRO_PID_SUBSCRIBER_COUNT, GYRO_PID_PUBLISHER_COUNT>;
pub type GyroPidSubscriber<'a> =
    Subscriber<'a, CriticalSectionRawMutex, GyroPidItem, 4, MAX_GYRO_PID_SUBSCRIBER_COUNT, GYRO_PID_PUBLISHER_COUNT>;

/// Macro to generate the `GlobalConfig` struct.<br>
/// Creates a new function that calls the new function of all the member configs.<br>
/// Also generates data-carrying enums for the `config` and `gyro_pid` `PubSubChannel`s.<br>
/// All member configs must define a static `new` function.<br>
macro_rules! define_configs {
    (
        general: [ $( $(#[$g_meta:meta])* ($g_enum:ident, $g_field:ident, $g_type:ty)),* $(,)?],
        gyro_pid: [ $( $(#[$p_meta:meta])* ($p_enum:ident, $p_field:ident, $p_type:ty)),* $(,)?]
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

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
        pub enum ConfigItem {
            $(
                $(#[$g_meta])*
                $g_enum($g_type)
            ),*
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
        pub enum GyroPidItem {
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
        (BatteryConfig, battery, BatteryConfig),

        #[cfg(feature = "blackbox")]
        (Blackbox, blackbox, BlackboxConfig),

        #[cfg(feature = "vtx")]
        (Vtx, vtx, VtxConfig),

        #[cfg(feature = "gps")]
        (Gps, gps, GpsConfig),

        #[cfg(feature = "osd")]
        (Osd, osd, OsdConfig),
        #[cfg(feature = "osd")]
        (OsdElements, osd_elements, OsdElementsConfig),
        #[cfg(feature = "osd")]
        (OsdStats, osd_stats, OsdStatsConfig),
    ],
    // GyroPid configs are defined separately so they can have their own PubSub channel.
    gyro_pid: [
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
    use serde::{Deserialize, Serialize};

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn _is_config<
        T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >() {
    }
    fn is_config_no_default<
        T: Sized + Send + Sync + Unpin + Copy + Clone + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >() {
    }

    #[test]
    fn normal_types() {
        is_full::<GlobalConfig>();
        is_config_no_default::<ConfigItem>();
        is_config_no_default::<GyroPidItem>();
    }
    #[test]
    fn global_config_new() {
        let _config = GlobalConfig::new();
    }
}
