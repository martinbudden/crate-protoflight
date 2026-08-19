use crate::gps::nmea_parser::{NmeaFields, parse_fixed_point, parse_int, parse_nmea_coordinate};

/// For GGA the fields are:
/// 0.  GPGGA
/// 1.  UTC time
/// 2.  latitude
/// 3.  N/S
/// 4.  longitude
/// 5.  E/W
/// 6.  fix quality
/// 7.  satellites
/// 8.  HDOP
/// 9.  altitude
/// 10. altitude units
/// 11. geoid separation
/// 12. geoid separation units
/// 13. DGPS age
/// 14. DGPS station ID
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NmeaGga {
    pub time_of_day_ms: u32,
    pub latitude_degrees_x1e7: i32,
    pub longitude_degrees_x1e7: i32,
    pub fix: u8,
    pub satellite_count: u8,
    pub hdop_x100: i16,
    pub altitude_cm: i32,
    pub geoid_separation_cm: i32,
}

impl Default for NmeaGga {
    fn default() -> Self {
        Self::new()
    }
}
#[allow(unused)]
impl NmeaGga {
    pub const fn new() -> Self {
        Self {
            time_of_day_ms: 0,
            latitude_degrees_x1e7: 0,
            longitude_degrees_x1e7: 0,
            fix: 0,
            satellite_count: 0,
            hdop_x100: 0,
            altitude_cm: 0,
            geoid_separation_cm: 0,
        }
    }
}

impl NmeaGga {
    pub fn parse(record: &[u8]) -> Option<Self> {
        let mut fields = NmeaFields::new(record);

        let talker_id = fields.next()?;

        if talker_id.len() != 5 || &talker_id[2..] != b"GGA" {
            return None;
        }

        let mut ret = Self::default();

        // Field 1: UTC time
        let _time = fields.next()?;
        //ret.time_of_day_ms = parse_nmea_time(time)?;

        // Field 2/3: latitude and N/S
        let latitude = fields.next()?;
        let latitude_direction = fields.next()?;

        // Field 4/5: longitude and E/W
        let longitude = fields.next()?;
        let longitude_direction = fields.next()?;

        if latitude_direction.len() != 1 || longitude_direction.len() != 1 {
            return None;
        }

        ret.latitude_degrees_x1e7 = parse_nmea_coordinate(latitude, latitude_direction[0])?;

        ret.longitude_degrees_x1e7 = parse_nmea_coordinate(longitude, longitude_direction[0])?;

        // Field 6: Fix Quality
        let raw = fields.next()?;
        let val = parse_int(raw)?;
        ret.fix = u8::try_from(val).ok()?;

        // Field 7: Satellites Tracked
        let raw = fields.next()?;
        let val = parse_int(raw)?;
        ret.satellite_count = u8::try_from(val).ok()?;

        // Field 8: HDOP
        let hdop = fields.next()?;
        let hdop = parse_fixed_point(hdop, 10)?;
        if hdop < 0 {
            return None;
        }
        ret.hdop_x100 = i16::try_from(hdop).ok()?;

        // Field 9/10: Altitude and units
        let altitude = fields.next()?;
        let altitude_units = fields.next()?;
        if altitude_units != b"M" {
            return None;
        }
        ret.altitude_cm = parse_fixed_point(altitude, 100)?;

        // Field 11: Geoid separation
        let geoid_separation = fields.next()?;
        let geoid_separation_units = fields.next()?;
        if geoid_separation_units != b"M" {
            return None;
        }
        ret.geoid_separation_cm = parse_fixed_point(geoid_separation, 100)?;

        // Fields 13 and 14: DGPS age and station ID.
        // Not currently used.
        Some(ret)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn parse_gga_record_extracts_position_and_fix() {
        let record = b"GPGGA,123519.500,4916.45,N,12311.12,W,1,08,0.9,545.4,M,46.9,M,,";

        let result = NmeaGga::parse(record).expect("GGA record should parse");

        //assert_eq!(result.time_of_week_ms, 45_319_500);

        assert_eq!(result.latitude_degrees_x1e7, 492_741_667);

        assert_eq!(result.longitude_degrees_x1e7, -1_231_853_333);

        assert_eq!(result.altitude_cm, 54_540);

        assert_eq!(result.hdop_x100, 9);

        assert_eq!(result.fix, 1);
        assert_eq!(result.satellite_count, 8);
        assert_eq!(result.geoid_separation_cm, 4_690);
    }
    #[test]
    fn parse_gga_record_rejects_invalid_latitude() {
        let record = b"GPGGA,123519,4916.X,N,12311.12,W,1,08,0.9,545.4,M,46.9,M,,";

        assert_eq!(NmeaGga::parse(record), None);
    }
    #[test]
    fn parse_gga_record_rejects_invalid_fix() {
        let record = b"GPGGA,123519,4916.45,N,12311.12,W,X,08,0.9,545.4,M,46.9,M,,";

        assert_eq!(NmeaGga::parse(record), None);
    }
    #[test]
    fn parse_gga_record_handles_negative_geoid_separation() {
        let record = b"GPGGA,123519.500,4916.45,N,12311.12,W,1,08,0.9,545.4,M,-46.9,M,,";

        let result = NmeaGga::parse(record).expect("GGA record should parse");

        assert_eq!(result.altitude_cm, 54_540);
        assert_eq!(result.geoid_separation_cm, -4_690);
    }
}
