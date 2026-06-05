#![allow(unused)]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct PidConfig {
    pub kp: u8, // proportional gain
    pub ki: u8, // integral gain
    pub kd: u8, // derivative gain
    pub ks: u8, // setpoint gain
    pub kk: u8, // setpoint derivative gain ('kick')
}

impl PidConfig {
    pub const fn new() -> Self {
        Self { kp: 0, ki: 0, kd: 0, ks: 0, kk: 0 }
    }
    pub const fn new5(kp: u8, ki: u8, kd: u8, ks: u8, kk: u8) -> Self {
        Self { kp, ki, kd, ks, kk }
    }
}

impl PostcardValue<'_> for PidConfig {}

impl Default for PidConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct PidConfigs {
    pub roll_rate: PidConfig,
    pub pitch_rate: PidConfig,
    pub yaw_rate: PidConfig,
    pub roll_angle: PidConfig,
    pub pitch_angle: PidConfig,
}

impl PidConfigs {
    pub const fn new() -> Self {
        Self {
            // Betaflight compatible defaults.
            roll_rate: PidConfig::new5(45, 80, 30, 120, 0),
            pitch_rate: PidConfig::new5(47, 84, 34, 125, 0),
            yaw_rate: PidConfig::new5(45, 80, 0, 120, 0),
            roll_angle: PidConfig::new5(50, 75, 75, 50, 0),
            pitch_angle: PidConfig::new5(50, 75, 75, 50, 0),
        }
    }
}

impl Default for PidConfigs {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration data for the flight controller filters.
/// These the dterm filters, the output filters, and the RC smoothing filters.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct FlightControllerFiltersConfig {
    //enum { PT1 : 0, BIQUAD, PT2, PT3 }
    pub dterm_lpf1_hz: u16,
    pub dterm_lpf2_hz: u16,
    //#if defined(USE_DTERM_FILTERS_EXTENDED)
    pub dterm_notch_hz: u16,
    pub dterm_notch_cutoff: u16,
    pub dterm_dynamic_lpf1_min_hz: u16,
    pub dterm_dynamic_lpf1_max_hz: u16,
    pub dterm_lpf1_type: u8,
    pub dterm_lpf2_type: u8,
    //#endif
    pub yaw_lpf_hz: u16,
    pub output_lpf_hz: u16,
    pub rc_smoothing_feedforward_cutoff: u8,
}

impl FlightControllerFiltersConfig {
    pub const PT1: u8 = 0;
    pub const BIQUAD: u8 = 1;
    pub const PT2: u8 = 2;
    pub const PT3: u8 = 3;

    pub const fn new() -> Self {
        Self {
            dterm_lpf1_hz: 75,
            dterm_lpf2_hz: 150,
            dterm_notch_hz: 0,
            dterm_notch_cutoff: 160,
            dterm_dynamic_lpf1_min_hz: 75,
            dterm_dynamic_lpf1_max_hz: 150,
            dterm_lpf1_type: Self::PT1,
            dterm_lpf2_type: Self::PT1,
            yaw_lpf_hz: 100,
            output_lpf_hz: 500,
            rc_smoothing_feedforward_cutoff: 0,
        }
    }
}

impl PostcardValue<'_> for FlightControllerFiltersConfig {}

impl Default for FlightControllerFiltersConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct FlightModeConfig {
    pub level_race_mode: u8, // aka "NFE(not fast enough) race mode": angle mode on roll, acro mode on pitch
}

impl FlightModeConfig {
    pub const fn new() -> Self {
        Self { level_race_mode: 0 }
    }
}

impl PostcardValue<'_> for FlightModeConfig {}

impl Default for FlightModeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration data for Throttle PID Attenuation (TPA),
/// Allows dynamic adjustment of the PID gains according to the throttle value.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct TpaConfig {
    pub mode: u8,
    pub rate: u8,
    pub breakpoint: u16,
    pub low_rate: i8,
    pub low_always: u8,
    pub low_breakpoint: u16,
}

impl TpaConfig {
    pub const TPA_MODE_P: u8 = 0;
    pub const TPA_MODE_D: u8 = 1;
    pub const TPA_MODE_PDS: u8 = 2;

    pub const fn new() -> Self {
        Self { mode: Self::TPA_MODE_D, rate: 65, breakpoint: 1350, low_rate: 20, low_always: 0, low_breakpoint: 1050 }
    }
}

impl PostcardValue<'_> for TpaConfig {}

impl Default for TpaConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct AntiGravityConfig {
    pub cutoff_hz: u8,
    pub p_gain: u8,
    pub i_gain: u8,
}

impl AntiGravityConfig {
    pub const fn new() -> Self {
        Self { cutoff_hz: 5, p_gain: 100, i_gain: 80 }
    }
}

impl PostcardValue<'_> for AntiGravityConfig {}

impl Default for AntiGravityConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct CrashFlipConfig {
    pub motor_percent: u8,
    pub rate: u8,
    pub auto_rearm: u8,
}

impl CrashFlipConfig {
    pub const fn new() -> Self {
        Self { motor_percent: 0, rate: 0, auto_rearm: 0 }
    }
}

impl PostcardValue<'_> for CrashFlipConfig {}

impl Default for CrashFlipConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct YawSpinRecoveryConfig {
    pub yaw_spin_threshold: i16,
    pub yaw_spin_recovery: u8,
}

impl YawSpinRecoveryConfig {
    pub const RECOVERY_OFF: u8 = 0;
    pub const RECOVERY_ON: u8 = 1;
    pub const RECOVERY_AUTO: u8 = 2;

    pub const fn new() -> Self {
        Self { yaw_spin_threshold: 0, yaw_spin_recovery: Self::RECOVERY_OFF }
    }
}

impl PostcardValue<'_> for YawSpinRecoveryConfig {}

impl Default for YawSpinRecoveryConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct CrashRecoveryConfig {
    pub d_threshold: u16,        // dterm crash value
    pub g_threshold: u16,        // gyro crash value
    pub setpoint_threshold: u16, // setpoint must be below this value to detect crash, so flips and rolls are not interpreted as crashes
    pub time: u16,               // ms
    pub delay: u16,              // ms
    pub limit_yaw: u16,          // limits yaw error rate, so crashes don't cause huge throttle increase
    pub recovery_angle: u8,      // degrees
    pub recovery_rate: u8,       // degrees per second
    pub recovery: u8,            // off, on, on and beeps when it is in crash recovery mode
}

impl CrashRecoveryConfig {
    pub const fn new() -> Self {
        Self {
            d_threshold: 50,         // dterm crash value
            g_threshold: 400,        // gyro crash value
            setpoint_threshold: 350, // setpoint must be below this value to detect crash, so flips and rolls are not interpreted as crashes
            time: 500,               // ms
            delay: 0,                // ms
            limit_yaw: 200,          // limits yaw error rate, so crashes don't cause huge throttle increase
            recovery_angle: 10,      // degrees
            recovery_rate: 100,      // degrees per second
            recovery: 0,             // off, on, on and beeps when it is in crash recovery mode
        }
    }
}

impl PostcardValue<'_> for CrashRecoveryConfig {}

impl Default for CrashRecoveryConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct ItermRelaxConfig {
    pub relax_type: u8,                   // not used
    pub relax: u8,                        // Enable iterm suppression during stick input
    pub relax_setpoint_threshold_dps: u8, // Full iterm suppression once setpoint has exceeded this value (degrees per second)
    pub relax_cutoff: u8, // Cutoff frequency used by low pass filter which predicts average response of the quad to setpoint
}

impl ItermRelaxConfig {
    #[allow(unused)]
    pub const RELAX_OFF: u8 = 0;
    pub const RELAX_ON: u8 = 1;
    pub const fn new() -> Self {
        Self { relax_type: 0, relax: Self::RELAX_ON, relax_setpoint_threshold_dps: 40, relax_cutoff: 15 }
    }
}

impl PostcardValue<'_> for ItermRelaxConfig {}

impl Default for ItermRelaxConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct DMaxConfig {
    pub d_max: [u8; 2], // Maximum D value on each axis
    pub gain: u8,       // gain factor for amount of gyro / setpoint activity required to boost D
    pub advance: u8,    // percentage multiplier for setpoint
}

impl DMaxConfig {
    pub const fn new() -> Self {
        Self { d_max: [0u8; 2], gain: 0, advance: 0 }
    }
}

impl PostcardValue<'_> for DMaxConfig {}

impl Default for DMaxConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct GyroConfig {
    pub gyro_movement_calibration_threshold: u8, // people keep forgetting that moving model while init results in wrong gyro offsets. and then they never reset gyro. so this is now on by default.
    pub gyro_hardware_lpf: u8,                   // gyro DLPF setting
    pub gyro_high_fsr: u8,

    pub gyro_lpf1_static_hz: u16,
    pub gyro_lpf2_static_hz: u16,

    pub gyro_soft_notch_hz_1: u16,
    pub gyro_soft_notch_cutoff_1: u16,
    pub gyro_soft_notch_hz_2: u16,
    pub gyro_soft_notch_cutoff_2: u16,
    pub gyro_offset_yaw: i16,
    pub check_overflow: u8,

    // Lowpass primary/secondary
    pub gyro_lpf1_type: u8,
    pub gyro_lpf2_type: u8,

    pub yaw_spin_recovery: u8,
    pub yaw_spin_threshold: i16,

    pub gyro_calibration_duration: u16, // Gyro calibration duration in 1/100 second

    pub gyro_lpf1_dyn_min_hz: u16,
    pub gyro_lpf1_dyn_max_hz: u16,

    pub gyro_filter_debug_axis: u8,

    pub gyros_detected: u8,     // What gyros should be shown as part of the cli status command.
    pub gyro_lpf1_dyn_expo: u8, // set the curve for dynamic gyro lowpass filter
    pub simplified_gyro_filter: u8,
    pub simplified_gyro_filter_multiplier: u8,

    pub gyro_enabled_bitmask: u8,
}

impl GyroConfig {
    pub const fn new() -> Self {
        Self {
            gyro_movement_calibration_threshold: 0, // people keep forgetting that moving model while init results in wrong gyro offsets. and then they never reset gyro. so this is now on by default.
            gyro_hardware_lpf: 0,                   // gyro DLPF setting
            gyro_high_fsr: 0,

            gyro_lpf1_static_hz: 0,
            gyro_lpf2_static_hz: 0,

            gyro_soft_notch_hz_1: 0,
            gyro_soft_notch_cutoff_1: 0,
            gyro_soft_notch_hz_2: 0,
            gyro_soft_notch_cutoff_2: 0,
            gyro_offset_yaw: 0,
            check_overflow: 0,

            // Lowpass primary/secondary
            gyro_lpf1_type: 0,
            gyro_lpf2_type: 0,

            yaw_spin_recovery: 0,
            yaw_spin_threshold: 0,

            gyro_calibration_duration: 0, // Gyro calibration duration in 1/100 second

            gyro_lpf1_dyn_min_hz: 0,
            gyro_lpf1_dyn_max_hz: 0,

            gyro_filter_debug_axis: 0,

            gyros_detected: 0,     // What gyros should be shown as part of the cli status command.
            gyro_lpf1_dyn_expo: 0, // set the curve for dynamic gyro lowpass filter
            simplified_gyro_filter: 0,
            simplified_gyro_filter_multiplier: 0,

            gyro_enabled_bitmask: 0,
        }
    }
}

impl PostcardValue<'_> for GyroConfig {}

impl Default for GyroConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<PidConfig>();
        is_full::<FlightControllerFiltersConfig>();
        is_full::<FlightModeConfig>();
        is_full::<TpaConfig>();
        is_full::<CrashFlipConfig>();
        is_full::<AntiGravityConfig>();
        is_full::<CrashFlipConfig>();
        is_full::<YawSpinRecoveryConfig>();
        is_full::<CrashRecoveryConfig>();
        is_full::<ItermRelaxConfig>();
        is_full::<DMaxConfig>();

        is_config::<PidConfig>();
        is_config::<FlightControllerFiltersConfig>();
        is_config::<FlightModeConfig>();
        is_config::<TpaConfig>();
        is_config::<CrashFlipConfig>();
        is_config::<AntiGravityConfig>();
        is_config::<CrashFlipConfig>();
        is_config::<YawSpinRecoveryConfig>();
        is_config::<CrashRecoveryConfig>();
        is_config::<ItermRelaxConfig>();
        is_config::<DMaxConfig>();
    }
    #[test]
    fn test_new() {
        let config = FlightControllerFiltersConfig::new();
        assert_eq!(75, config.dterm_lpf1_hz);
    }
}
