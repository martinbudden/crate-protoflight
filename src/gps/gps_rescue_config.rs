#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

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
    pub sanity_checks: GpsRescueSanityChecks,
    pub allow_arming_without_fix: u8,
    pub use_mag: u8,
    pub altitude_mode: GpsRescueAltitudeMode,
    pub ascend_rate: u16,
    pub descend_rate: u16,
    pub initial_climb_m: u16,
    pub roll_mix: u8,
    pub disarm_threshold: u8,
    pub pitch_cutoff_hz: u8,
    pub imu_yaw_gain: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for GpsRescueConfig {}

impl Default for GpsRescueConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl GpsRescueConfig {
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
            sanity_checks: GpsRescueSanityChecks::FsOnly,
            allow_arming_without_fix: 0,
            use_mag: 0,
            altitude_mode: GpsRescueAltitudeMode::Max,
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

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GpsRescueSanityChecks {
    Off = 0,
    On = 1,
    #[default]
    FsOnly = 2,
}

#[allow(unused)]
impl GpsRescueSanityChecks {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Off,
            1 => Self::On,
            2 => Self::FsOnly,
            _ => Self::default(),
        }
    }
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GpsRescueAltitudeMode {
    #[default]
    Max = 0,
    Fixed = 1,
    Current = 2,
}

#[allow(unused)]
impl GpsRescueAltitudeMode {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Max,
            1 => Self::Fixed,
            2 => Self::Current,
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
        is_full::<GpsRescueConfig>();
        #[cfg(feature = "serde")]
        is_config::<GpsRescueConfig>();
    }
}
