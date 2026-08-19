use crate::gps::{NmeaRmc, nmea_gga::NmeaGga, nmea_gsa::NmeaGsa, nmea_gsv::NmeaGsv, ubx_nav_pvt_data::NavPvtData};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpsData {
    pub longitude_degrees_x1e7: i32,
    pub latitude_degrees_x1e7: i32,
    pub altitude_cm: i32,
    pub geoid_separation_cm: i32,
    pub distance_to_home_meters: f32,
    pub bearing_to_home_degrees: f32,
    pub distance_flown_meters: f32,
    pub time_of_week_ms: u32,
    pub velocity_north_cmps: i32,
    pub velocity_east_cmps: i32,
    pub velocity_down_cmps: i32,
    pub speed3d_cmps: i32,
    pub ground_speed_cmps: i16,
    pub heading_deci_degrees: i16,
    pub satellite_count: u8, // GGA: satellites tracked
    pub satellites_used: u8, // GSA: satellites used in solution
    pub fix: u8,             // GGA fix quality
    pub fix_type: u8,        // GSA: 1/2/3
    pub is_healthy: u8,

    pub dilution_of_precision_horizontal: i16, // GGA
    pub dilution_of_precision_position: i16,   // GSA
    pub dilution_of_precision_vertical: i16,   // GSA
    pub update: u8,
}

impl Default for GpsData {
    fn default() -> Self {
        Self::new()
    }
}
#[allow(unused)]
impl GpsData {
    const FIX_HOME: u8 = 0x01;
    const FIX: u8 = 0x02;
    const FIX_EVER: u8 = 0x04;

    pub const fn new() -> Self {
        Self {
            longitude_degrees_x1e7: 0,
            latitude_degrees_x1e7: 0,
            altitude_cm: 0,
            geoid_separation_cm: 0,
            distance_to_home_meters: 0.0,
            bearing_to_home_degrees: 0.0,
            distance_flown_meters: 0.0,
            time_of_week_ms: 0,
            velocity_north_cmps: 0,
            velocity_east_cmps: 0,
            velocity_down_cmps: 0,
            speed3d_cmps: 0,
            ground_speed_cmps: 0,
            heading_deci_degrees: 0,
            satellite_count: 0,
            satellites_used: 0,
            fix: 0,
            fix_type: 0,
            is_healthy: 0,
            dilution_of_precision_horizontal: 0,
            dilution_of_precision_position: 0,
            dilution_of_precision_vertical: 0,
            update: 0,
        }
    }
}

impl GpsData {
    pub fn amend_with_gga(&mut self, gga: NmeaGga) {
        self.latitude_degrees_x1e7 = gga.latitude_degrees_x1e7;
        self.longitude_degrees_x1e7 = gga.longitude_degrees_x1e7;
        self.altitude_cm = gga.altitude_cm;
        self.geoid_separation_cm = gga.geoid_separation_cm;
        self.satellite_count = gga.satellite_count;
        self.fix = gga.fix;
        // ...
    }

    // TODO: amend_with_gsa
    pub fn amend_with_gsa(&mut self, gsa: NmeaGsa) {
        _ = self;
        _ = gsa;
    }

    // TODO: amend_with_gsa
    #[allow(unused)]
    pub fn amend_with_gsv(&mut self, gsv: NmeaGsv) {
        _ = self;
        _ = gsv;
    }

    // TODO: amend_with_rmc
    pub fn amend_with_rmc(&mut self, gsv: NmeaRmc) {
        _ = self;
        _ = gsv;
    }

    pub fn amend_with_nav_pvt_data(&mut self, nav: NavPvtData) {
        self.longitude_degrees_x1e7 = nav.longitude_degrees_x1e7;
        self.latitude_degrees_x1e7 = nav.latitude_degrees_x1e7;
        self.altitude_cm = nav.height_msl_mm / 10;

        let geoid_separation_mm = nav.height_ellipsoid_mm - nav.height_msl_mm;
        self.geoid_separation_cm = geoid_separation_mm / 10;

        self.time_of_week_ms = nav.time_of_week_ms;

        self.velocity_north_cmps = nav.velocity_north_mmps / 10;
        self.velocity_east_cmps = nav.velocity_east_mmps / 10;
        self.velocity_down_cmps = nav.velocity_down_mmps / 10;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        {
            self.ground_speed_cmps =
                (nav.ground_speed_mmps / 10).clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
            self.heading_deci_degrees =
                (nav.heading_motion_x1e5_deg as i32 / 1000).clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
        }

        self.satellite_count = nav.satellite_count;
        self.fix = nav.fix_type;
        self.is_healthy = u8::from(nav.flags & 0x01 != 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<GpsData>();
    }
    #[test]
    fn nav_pvt_maps_time_of_week() {
        let nav = NavPvtData { time_of_week_ms: 123_456_789, ..NavPvtData::default() };

        let mut gps = GpsData::default();
        gps.amend_with_nav_pvt_data(nav);

        assert_eq!(gps.time_of_week_ms, 123_456_789);
    }
    #[test]
    fn nav_pvt_maps_fix_and_health() {
        let nav = NavPvtData { fix_type: 3, flags: 0x01, ..NavPvtData::default() };

        let mut gps = GpsData::default();
        gps.amend_with_nav_pvt_data(nav);

        assert_eq!(gps.fix, 3);
        assert_eq!(gps.is_healthy, 1);
    }
    #[test]
    fn nav_pvt_fix_is_unhealthy_when_gnss_fix_ok_is_clear() {
        let nav = NavPvtData { fix_type: 3, flags: 0x00, ..NavPvtData::default() };

        let mut gps = GpsData::default();
        gps.amend_with_nav_pvt_data(nav);

        assert_eq!(gps.fix, 3);
        assert_eq!(gps.is_healthy, 0);
    }
}
