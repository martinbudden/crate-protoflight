use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
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

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    #[allow(unused)]
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_config<
        T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >() {
    }

    #[test]
    fn normal_types() {
        is_config::<GpsConfig>();
        is_full::<Gps>();
    }
    #[test]
    fn test_new() {
        let config = GpsConfig::new();
        assert_eq!(1, config.provider);
    }
}
