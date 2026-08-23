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

impl Default for GpsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl GpsConfig {
    pub const fn new() -> Self {
        Self {
            provider: GpsProvider::Ubx,
            sbas_mode: SbasMode::SbasNone,
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

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GpsProvider {
    #[default]
    Nmea = 0,
    Ubx = 1,
    Msp = 2,
    Mock = 3,
    NoGps = 255,
}

#[allow(unused)]
impl GpsProvider {
    /// Forgiving conversion, converts invalid values to default.
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Nmea,
            1 => Self::Ubx,
            2 => Self::Msp,
            3 => Self::Mock,
            255 => Self::NoGps,
            _ => Self::default(),
        }
    }
}

#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
    /// Forgiving conversion, converts invalid values to default.
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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
    /// Forgiving conversion, converts invalid values to default.
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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SbasMode {
    #[default]
    Auto = 0,
    Egnos = 1,
    Waas = 2,
    Msas = 3,
    Gagan = 4,
    SbasNone = 5,
}

#[allow(unused)]
impl SbasMode {
    /// Forgiving conversion, converts invalid values to default.
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Auto,
            1 => Self::Egnos,
            2 => Self::Waas,
            3 => Self::Gagan,
            4 => Self::SbasNone,
            _ => Self::default(),
        }
    }
}

#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GpsOffOn {
    #[default]
    Off = 0,
    On = 1,
}

#[allow(unused)]
impl GpsOffOn {
    /// Forgiving conversion, converts invalid values to default.
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

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + Eq + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<GpsConfig>();
        #[cfg(feature = "serde")]
        is_config::<GpsConfig>();
        is_full_eq::<GpsProvider>();
        is_full_eq::<GpsModel>();
        is_full_eq::<UtcStandard>();
        is_full_eq::<SbasMode>();
        is_full_eq::<GpsOffOn>();
    }
    #[test]
    fn test_new() {
        let config = GpsConfig::new();
        assert_eq!(GpsProvider::Ubx, config.provider);
    }
}
