#![cfg(feature = "gps")]

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GpsConfig {
    pub provider: u8,
    pub sbas_mode: u8,
    pub auto_config: u8,
    pub auto_baud: u8,
    pub gps_ublox_acquire_model: u8,
    pub gps_ublox_flight_model: u8,
    pub gps_update_rate_hz: u8,
    pub gps_ublox_use_galileo: u8,
    pub gps_set_home_point_once: u8,
    pub gps_use_3d_speed: u8,
    pub sbas_integrity: u8,
    pub gps_ublox_utc_standard: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for GpsConfig {}

impl GpsConfig {
    pub const fn new() -> Self {
        Self {
            provider: Gps::GPS_UBLOX,
            sbas_mode: Gps::SBAS_NONE,
            auto_config: Gps::AUTO_CONFIG_ON,
            auto_baud: Gps::AUTO_BAUD_OFF,
            gps_ublox_acquire_model: Gps::MODEL_STATIONARY,
            gps_ublox_flight_model: Gps::MODEL_AIRBORNE_4G,
            gps_update_rate_hz: 10,
            gps_ublox_use_galileo: 0,
            gps_set_home_point_once: 0,
            gps_use_3d_speed: 0,
            sbas_integrity: 0,
            gps_ublox_utc_standard: Gps::UTC_STANDARD_AUTO,
        }
    }
}

impl Default for GpsConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GpsRescueConfig {
    pub max_rescue_angle_degrees: u16,
    pub return_altitude_m: u16,
    pub descent_distance_m: u16,
    pub ground_speed_cmps: u16,
    pub yaw_kp: u8,
    pub min_sats: u8,
    pub vel_kp: u8,
    pub vel_ki: u8,
    pub vel_kd: u8,
    pub min_start_dist_m: u16,
    pub sanity_checks: u8,
    pub allow_arming_without_fix: u8,
    pub use_mag: u8,
    pub altitude_mode: u8,
    pub ascend_rate: u16,
    pub descend_rate: u16,
    pub initial_climb_m: u16,
    pub roll_mix: u8,
    pub disarm_threshold: u8,
    pub pitch_cutoff_hz: u8,
    pub imu_yaw_gain: u8,
}

#[allow(unused)]
impl GpsRescueConfig {
    const SANITY_OFF: u8 = 0;
    const SANITY_ON: u8 = 1;
    const SANITY_FS_ONLY: u8 = 2;

    const ALT_MODE_MAX: u8 = 0;
    const ALT_MODE_FIXED: u8 = 1;
    const ALT_MODE_CURRENT: u8 = 2;
    pub const fn new() -> Self {
        Self {
            max_rescue_angle_degrees: 45,
            return_altitude_m: 30,
            descent_distance_m: 20,
            ground_speed_cmps: 750,
            yaw_kp: 20,
            min_sats: 8,
            vel_kp: 8,
            vel_ki: 40,
            vel_kd: 12,
            min_start_dist_m: 15,
            sanity_checks: Self::SANITY_FS_ONLY,
            allow_arming_without_fix: 0,
            use_mag: 0,
            altitude_mode: Self::ALT_MODE_MAX,
            ascend_rate: 750,
            descend_rate: 150,
            initial_climb_m: 10,
            roll_mix: 150,
            disarm_threshold: 30,
            pitch_cutoff_hz: 75,
            imu_yaw_gain: 10,
        }
    }
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for GpsRescueConfig {}

impl Default for GpsRescueConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Gps {}

impl Gps {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for Gps {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl Gps {
    const MODEL_PORTABLE: u8 = 0;
    const MODEL_STATIONARY: u8 = 1;
    const MODEL_PEDESTRIAN: u8 = 2;
    const MODEL_AUTOMOTIVE: u8 = 3;
    const MODEL_AT_SEA: u8 = 4;
    const MODEL_AIRBORNE_1G: u8 = 5;
    const MODEL_AIRBORNE_2G: u8 = 6;
    const MODEL_AIRBORNE_4G: u8 = 7;
    const UTC_STANDARD_AUTO: u8 = 0;
    const UTC_STANDARD_USNO: u8 = 3;
    const UTC_STANDARD_EU: u8 = 5;
    const UTC_STANDARD_SU: u8 = 6;
    const UTC_STANDARD_NTSC: u8 = 7;

    const SBAS_AUTO: u8 = 0;
    const SBAS_EGNOS: u8 = 1;
    const SBAS_WAAS: u8 = 2;
    const SBAS_MSAS: u8 = 3;
    const SBAS_GAGAN: u8 = 4;
    const SBAS_NONE: u8 = 5;

    const AUTO_CONFIG_OFF: u8 = 0;
    const AUTO_CONFIG_ON: u8 = 1;
    const AUTO_BAUD_OFF: u8 = 0;
    const AUTO_BAUD_ON: u8 = 1;

    const GPS_NMEA: u8 = 0;
    const GPS_UBLOX: u8 = 1;
    const GPS_MSP: u8 = 2;
    const GPS_VIRTUAL: u8 = 3;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<GpsConfig>();
        #[cfg(feature = "serde")]
        is_config::<GpsConfig>();
        is_full::<GpsRescueConfig>();
        #[cfg(feature = "serde")]
        is_config::<GpsRescueConfig>();
        is_full::<Gps>();
    }
    #[test]
    fn test_new() {
        let config = GpsConfig::new();
        assert_eq!(1, config.provider);
    }
}
