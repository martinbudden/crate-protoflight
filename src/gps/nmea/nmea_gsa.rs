use super::nmea_parser::{NmeaFields, Parse};

/// `GPGSA,A,3,04,05,09,12,24,25,29,31,,,,,1.8,1.0,1.5`.
/// |    # | Field         | Example     | Meaning             |
/// | ---: | ------------- | ----------- | ------------------- |
/// |    0 | Talker/type   | `GPGSA`     | Message type        |
/// |    1 | Mode          | `A`         | Auto/manual         |
/// |    2 | Fix type      | `3`         | No fix / 2D / 3D    |
/// | 3–14 | Satellite IDs | `04,05,...` | Satellites used     |
/// |   15 | PDOP          | `1.8`       | Position dilution   |
/// |   16 | HDOP          | `1.0`       | Horizontal dilution |
/// |   17 | VDOP          | `1.5`       | Vertical dilution   |
/// |   18 | System ID     | optional    | GNSS constellation  |
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NmeaGsa {
    pub longitude_degrees_x1e7: i32,
    pub latitude_degrees_x1e7: i32,
    pub altitude_cm: i32,
    pub geoid_separation_cm: i32,
    pub distance_to_home_meters: f32,
    pub bearing_to_home_degrees: f32,
    pub distance_flown_meters: f32,
    pub time_of_week_ms: u32,
    pub velocity_north_cmps: i16,
    pub velocity_east_cmps: i16,
    pub velocity_down_cmps: i16,
    pub speed3d_cmps: i16,
    pub ground_speed_cmps: i16,
    pub heading_deci_degrees: i16,
    pub satellite_count: u8, // GGA: satellites tracked
    pub satellites_used: u8, // GSA: satellites used in solution
    pub fix: u8,             // GGA fix quality
    pub fix_type: u8,        // GSA: 1/2/3
    pub is_healthy: u8,

    pub hdop: u16,
    pub pdop: u16,
    pub vdop: u16,
    pub update: u8,
}

impl Default for NmeaGsa {
    fn default() -> Self {
        Self::new()
    }
}

impl NmeaGsa {
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
            hdop: 0,
            pdop: 0,
            vdop: 0,
            update: 0,
        }
    }
}

impl NmeaGsa {
    pub fn parse(record: &[u8]) -> Option<Self> {
        let mut fields = NmeaFields::new(record);

        // Field 0: Talker/type
        let talker_id = fields.next()?;
        if talker_id.len() != 5 || &talker_id[2..] != b"GSA" {
            return None;
        }

        let mut ret = Self::default();

        // Field 1: Mode
        let mode = fields.next()?;
        if mode.len() != 1 || (mode[0] != b'A' && mode[0] != b'M') {
            return None;
        }

        // Field 2: Fix type
        let raw = fields.next()?;
        let fix_type = Parse::int(raw)?;
        ret.fix_type = u8::try_from(fix_type).ok()?;
        if !(1..=3).contains(&ret.fix_type) {
            return None;
        }

        // Fields 3-14: Satellite IDs used in the solution.
        // Even if a satellite slot is empty, NmeaFields returns Some(&[]) because the empty field is between commas.
        // For example:
        // GPGSA,A,3,04,05,,12,,,,,,,,,1.8,1.0,1.5
        // will produce empty slices for the unused satellite positions, and we simply don't increment satellites_used.
        for _ in 0..12 {
            let satellite = fields.next()?;
            if !satellite.is_empty() {
                // Validate that the satellite ID is numeric and fits in u8.
                let satellite_id = Parse::int(satellite)?;
                _ = u8::try_from(satellite_id).ok()?;
                ret.satellites_used = ret.satellites_used.checked_add(1)?;
            }
        }

        // Field 15: PDOP
        let raw = fields.next()?;
        let pdop = Parse::fixed_point(raw, 10)?;
        if pdop < 0 {
            return None;
        }
        ret.pdop = u16::try_from(pdop).ok()?;

        // Field 16: HDOP
        let raw = fields.next()?;
        let hdop = Parse::fixed_point(raw, 10)?;
        if hdop < 0 {
            return None;
        }
        ret.hdop = u16::try_from(hdop).ok()?;

        // Field 17: VDOP
        let raw = fields.next()?;
        let vdop = Parse::fixed_point(raw, 10)?;
        if vdop < 0 {
            return None;
        }

        ret.vdop = u16::try_from(vdop).ok()?;

        // Field 18: GNSS system ID, if present.
        // Not currently used.

        Some(ret)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn parse_gsa_record_extracts_fix_and_dop() {
        let record = b"GPGSA,A,3,04,05,09,12,24,25,29,31,,,,,1.8,1.0,1.5";

        let result = NmeaGsa::parse(record).expect("GSA record should parse");

        assert_eq!(result.fix_type, 3);
        assert_eq!(result.satellites_used, 8);

        assert_eq!(result.pdop, 18);
        assert_eq!(result.hdop, 10);
        assert_eq!(result.vdop, 15);
    }
    #[test]
    fn parse_gsa_record_accepts_no_fix() {
        let record = b"GPGSA,A,1,,,,,,,,,,,,,1.8,1.0,1.5";

        let result = NmeaGsa::parse(record).expect("GSA record should parse");

        assert_eq!(result.fix_type, 1);
        assert_eq!(result.satellites_used, 0);
    }
    #[test]
    fn parse_gsa_record_rejects_invalid_fix_type() {
        let record = b"GPGSA,A,4,04,05,,,,,,,,,,,1.8,1.0,1.5";

        assert_eq!(NmeaGsa::parse(record), None);
    }
    #[test]
    fn parse_gsa_record_rejects_invalid_mode() {
        let record = b"GPGSA,X,3,04,05,,,,,,,,,,,1.8,1.0,1.5";

        assert_eq!(NmeaGsa::parse(record), None);
    }
}
