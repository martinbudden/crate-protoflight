use crate::gps::nmea_parser::{NmeaFields, Parse};

/// `GPRMC,123519.00,A,4916.45,N,12311.12,W,022.4,084.4,230394,,,A`.
///
/// |  # | Field              | Example     | Usage            |
/// | -: | ------------------ | ----------- | ---------------- |
/// |  0 | Talker/type        | `GPRMC`     | validate `RMC`   |
/// |  1 | UTC time           | `123519.00` | `time_of_day_ms` |
/// |  2 | Status             | `A`         | `is_healthy`     |
/// |  3 | Latitude           | `4916.45`   | latitude         |
/// |  4 | N/S                | `N`         | latitude sign    |
/// |  5 | Longitude          | `12311.12`  | longitude        |
/// |  6 | E/W                | `W`         | longitude sign   |
/// |  7 | Speed over ground  | `022.4`     | ground speed     |
/// |  8 | Course over ground | `084.4`     | heading          |
/// |  9 | Date               | `230394`    | currently ignore |
/// | 10 | Magnetic variation | optional    | ignore           |
/// | 11 | E/W                | optional    | ignore           |
/// | 12 | Mode               | optional    | ignore           |
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NmeaRmc {
    pub time_of_day_ms: u32,
    pub latitude_degrees_x1e7: i32,
    pub longitude_degrees_x1e7: i32,
    pub is_healthy: u8,
    pub ground_speed_cmps: i16,
    pub heading_deci_degrees: i16,
}

impl Default for NmeaRmc {
    fn default() -> Self {
        Self::new()
    }
}
#[allow(unused)]
impl NmeaRmc {
    pub const fn new() -> Self {
        Self {
            time_of_day_ms: 0,
            latitude_degrees_x1e7: 0,
            longitude_degrees_x1e7: 0,
            is_healthy: 0,
            ground_speed_cmps: 0,
            heading_deci_degrees: 0,
        }
    }
}

impl NmeaRmc {
    pub fn parse(record: &[u8]) -> Option<Self> {
        let mut fields = NmeaFields::new(record);

        let talker_id = fields.next()?;

        if talker_id.len() != 5 || &talker_id[2..] != b"RMC" {
            return None;
        }

        let mut ret = Self::default();

        // Field 1: UTC time
        let _time = fields.next()?;
        //ret.time_of_day_ms = parse_nmea_time(time)?;

        // Field 2: Status
        let status = fields.next()?;

        if status.len() != 1 {
            return None;
        }

        match status[0] {
            b'A' => ret.is_healthy = 1,
            b'V' => ret.is_healthy = 0,
            _ => return None,
        }

        // Field 3/4: latitude and N/S
        let latitude = fields.next()?;
        let latitude_direction = fields.next()?;

        // Field 5/6: longitude and E/W
        let longitude = fields.next()?;
        let longitude_direction = fields.next()?;

        if latitude_direction.len() != 1 || longitude_direction.len() != 1 {
            return None;
        }

        ret.latitude_degrees_x1e7 = Parse::nmea_coordinate(latitude, latitude_direction[0])?;

        ret.longitude_degrees_x1e7 = Parse::nmea_coordinate(longitude, longitude_direction[0])?;

        // Field 7: Speed over ground, knots
        let speed = fields.next()?;
        let speed_knots_x10 = Parse::fixed_point(speed, 10)?;
        // convert from knots to cm/s
        let speed_cmps = speed_knots_x10.checked_mul(1852)?.checked_div(360)?;

        // Field 8: Course over ground, degrees
        ret.ground_speed_cmps = i16::try_from(speed_cmps).ok()?;
        let course = fields.next()?;
        let course = Parse::fixed_point(course, 10)?;
        ret.heading_deci_degrees = i16::try_from(course).ok()?;

        // Field 9: Date
        let _date = fields.next()?;

        // Fields 10-12: Magnetic variation and mode.
        // Not currently used.

        Some(ret)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn parse_rmc_record_extracts_navigation_data() {
        let record = b"GPRMC,123519.00,A,4916.45,N,12311.12,W,022.4,084.4,230394,,,";

        let result = NmeaRmc::parse(record).expect("RMC record should parse");

        //assert_eq!(result.time_of_week_ms, 45_319_000);

        assert_eq!(result.latitude_degrees_x1e7, 492_741_667);

        assert_eq!(result.longitude_degrees_x1e7, -1_231_853_333);

        assert_eq!(result.ground_speed_cmps, 1_152);
        assert_eq!(result.heading_deci_degrees, 844);

        assert_eq!(result.is_healthy, 1);
    }
    #[test]
    fn parse_rmc_record_marks_invalid_fix_unhealthy() {
        let record = b"GPRMC,123519.00,V,4916.45,N,12311.12,W,022.4,084.4,230394,,,";

        let result = NmeaRmc::parse(record).expect("RMC record should parse");

        assert_eq!(result.is_healthy, 0);
    }
    #[test]
    fn parse_rmc_record_accepts_different_talker_ids() {
        let record = b"GNRMC,123519.00,A,4916.45,N,12311.12,W,022.4,084.4,230394,,,";

        assert!(NmeaRmc::parse(record).is_some());
    }
}
