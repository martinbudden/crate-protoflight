#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GpsConfig {
    pub provider: GpsProvider,
    pub sbas_mode: SbasMode,
    pub auto_config: GpsOffOn,
    pub auto_baud: GpsOffOn,
    pub gps_ublox_acquire_model: GpsModel,
    pub gps_ublox_flight_model: GpsModel,
    pub gps_update_rate_hz: u8,
    pub gps_ublox_use_galileo: u8,
    pub gps_set_home_point_once: u8,
    pub gps_use_3d_speed: u8,
    pub sbas_integrity: u8,
    pub gps_ublox_utc_standard: UtcStandard,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for GpsConfig {}

impl GpsConfig {
    pub const fn new() -> Self {
        Self {
            provider: GpsProvider::Ublox,
            sbas_mode: SbasMode::None,
            auto_config: GpsOffOn::On,
            auto_baud: GpsOffOn::Off,
            gps_ublox_acquire_model: GpsModel::Stationary,
            gps_ublox_flight_model: GpsModel::Airborne4G,
            gps_update_rate_hz: 10,
            gps_ublox_use_galileo: 0,
            gps_set_home_point_once: 0,
            gps_use_3d_speed: 0,
            sbas_integrity: 0,
            gps_ublox_utc_standard: UtcStandard::Auto,
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

#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GpsProvider {
    #[default]
    Nmea = 0,
    Ublox = 1,
    Msp = 2,
    Mock = 3,
    None = 255,
}

#[allow(unused)]
impl GpsProvider {
    pub const COUNT: u8 = 13;

    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Nmea,
            1 => Self::Ublox,
            2 => Self::Msp,
            3 => Self::Mock,
            255 => Self::None,
            _ => Self::default(),
        }
    }
}

#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GpsModel {
    #[default]
    Portable = 0,
    Stationary = 1,
    Pedestrian = 2,
    Automotive = 3,
    AtSea = 4,
    Airborne1G = 5,
    Airborne2G = 6,
    Airborne4G = 7,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for GpsModel {}

#[allow(unused)]
impl GpsModel {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Portable,
            1 => Self::Stationary,
            2 => Self::Pedestrian,
            3 => Self::Automotive,
            4 => Self::AtSea,
            5 => Self::Airborne1G,
            6 => Self::Airborne2G,
            7 => Self::Airborne4G,
            _ => Self::default(),
        }
    }
}

#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UtcStandard {
    #[default]
    Auto = 0,
    Usno = 3,
    Eu = 5,
    Su = 6,
    Ntsc = 7,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for UtcStandard {}

#[allow(unused)]
impl UtcStandard {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Auto,
            3 => Self::Usno,
            5 => Self::Eu,
            6 => Self::Su,
            7 => Self::Ntsc,
            _ => Self::default(),
        }
    }
}

#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SbasMode {
    #[default]
    Auto = 0,
    Egnos = 1,
    Waas = 2,
    Msas = 3,
    Gagan = 4,
    None = 5,
}

#[allow(unused)]
impl SbasMode {
    pub const COUNT: u8 = 13;

    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Auto,
            1 => Self::Egnos,
            2 => Self::Waas,
            3 => Self::Gagan,
            4 => Self::None,
            _ => Self::default(),
        }
    }
}

#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GpsOffOn {
    #[default]
    Off = 0,
    On = 1,
}

#[allow(unused)]
impl GpsOffOn {
    pub const COUNT: u8 = 2;

    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Off,
            1 => Self::On,
            _ => Self::default(),
        }
    }
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
        is_full::<GpsOffOn>();
    }
    #[test]
    fn test_new() {
        let config = GpsConfig::new();
        assert_eq!(GpsProvider::Ublox, config.provider);
    }
}
