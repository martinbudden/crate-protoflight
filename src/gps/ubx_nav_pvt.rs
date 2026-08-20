use crate::gps::{
    ubx_nav::{UBX_NAV_CLASS, UbxNavId},
    ubx_parser::Parse,
};

/*
The NAV-PVT payload has a fixed binary layout. The fields most relevant to our existing GpsData are:

| Offset | Size | Field                       | Useful to us |
| -----: | ---: | --------------------------- | ------------ |
|      0 |    4 | `iTOW`                      | Yes          |
|      4 |    2 | Year                        | Probably     |
|      6 |    1 | Month                       | Probably     |
|      7 |    1 | Day                         | Probably     |
|      8 |    1 | Hour                        | Yes          |
|      9 |    1 | Minute                      | Yes          |
|     10 |    1 | Second                      | Yes          |
|     11 |    1 | `valid` flags               | Yes          |
|     12 |    1 | `tAcc`                      | Maybe        |
|     16 |    4 | `nano`                      | Maybe        |
|     20 |    1 | `fixType`                   | Yes          |
|     21 |    1 | `flags`                     | Yes          |
|     23 |    1 | `numSV`                     | Yes          |
|     24 |    4 | longitude                   | Yes          |
|     28 |    4 | latitude                    | Yes          |
|     32 |    4 | height above ellipsoid      | Yes          |
|     36 |    4 | height above mean sea level | Yes          |
|     40 |    4 | horizontal accuracy         | Yes          |
|     44 |    4 | vertical accuracy           | Yes          |
|     48 |    4 | velocity north              | Yes          |
|     52 |    4 | velocity east               | Yes          |
|     56 |    4 | velocity down               | Yes          |
|     60 |    4 | ground speed                | Yes          |
|     64 |    4 | heading of motion           | Yes          |
*/
///Navigation position velocity time solution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxNavPvt {
    pub time_of_week_ms: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,

    pub valid: u8,
    pub time_accuracy_ns: u32,
    pub nano: i32,

    pub fix_type: u8,
    pub flags: u8,
    pub flags2: u8,
    pub satellite_count: u8,

    pub longitude_degrees_x1e7: i32,
    pub latitude_degrees_x1e7: i32,
    pub height_ellipsoid_mm: i32,
    pub height_msl_mm: i32,
    pub horizontal_accuracy_mm: u32,
    pub vertical_accuracy_mm: u32,

    pub velocity_north_mmps: i32,
    pub velocity_east_mmps: i32,
    pub velocity_down_mmps: i32,
    pub ground_speed_mmps: i32,
    pub heading_degrees_x1e5: u32,
}

impl Default for UbxNavPvt {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxNavPvt {
    pub const CLASS: u8 = UBX_NAV_CLASS;
    pub const ID: u8 = UbxNavId::PVT;
    pub const PAYLOAD_LEN: usize = 92;

    pub const fn new() -> Self {
        Self {
            time_of_week_ms: 0,
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
            valid: 0,
            time_accuracy_ns: 0,
            nano: 0,
            fix_type: 0,
            flags: 0,
            flags2: 0,
            satellite_count: 0,
            longitude_degrees_x1e7: 0,
            latitude_degrees_x1e7: 0,
            height_ellipsoid_mm: 0,
            height_msl_mm: 0,
            horizontal_accuracy_mm: 0,
            vertical_accuracy_mm: 0,
            velocity_north_mmps: 0,
            velocity_east_mmps: 0,
            velocity_down_mmps: 0,
            ground_speed_mmps: 0,
            heading_degrees_x1e5: 0,
        }
    }
}

impl UbxNavPvt {
    pub fn parse(payload: &[u8]) -> Option<UbxNavPvt> {
        if payload.len() != UbxNavPvt::PAYLOAD_LEN {
            return None;
        }
        Some(UbxNavPvt {
            time_of_week_ms: Parse::try_read_u32(&payload[0..4])?,

            year: Parse::try_read_u16(&payload[4..6])?,
            month: payload[6],
            day: payload[7],
            hour: payload[8],
            minute: payload[9],
            second: payload[10],

            valid: payload[11],
            time_accuracy_ns: Parse::try_read_u32(&payload[12..16])?,
            nano: Parse::try_read_i32(&payload[16..20])?,

            fix_type: payload[20],
            flags: payload[21],
            flags2: payload[22],
            satellite_count: payload[23],
            longitude_degrees_x1e7: Parse::try_read_i32(&payload[24..28])?,
            latitude_degrees_x1e7: Parse::try_read_i32(&payload[28..32])?,
            height_ellipsoid_mm: Parse::try_read_i32(&payload[32..36])?,
            height_msl_mm: Parse::try_read_i32(&payload[36..40])?,
            horizontal_accuracy_mm: Parse::try_read_u32(&payload[40..44])?,
            vertical_accuracy_mm: Parse::try_read_u32(&payload[44..48])?,
            velocity_north_mmps: Parse::try_read_i32(&payload[48..52])?,
            velocity_east_mmps: Parse::try_read_i32(&payload[52..56])?,
            velocity_down_mmps: Parse::try_read_i32(&payload[56..60])?,
            ground_speed_mmps: Parse::try_read_i32(&payload[60..64])?,
            heading_degrees_x1e5: Parse::try_read_u32(&payload[64..68])?,
        })
    }
}

pub(crate) fn make_realistic_nav_pvt_payload() -> [u8; UbxNavPvt::PAYLOAD_LEN] {
    let mut payload = [0u8; UbxNavPvt::PAYLOAD_LEN];

    // iTOW: 12:34:56.789
    payload[0..4].copy_from_slice(&45_296_789u32.to_le_bytes());

    // UTC: 2026-08-18 12:34:56
    payload[4..6].copy_from_slice(&2026u16.to_le_bytes());
    payload[6] = 8;
    payload[7] = 18;
    payload[8] = 12;
    payload[9] = 34;
    payload[10] = 56;

    // Time valid, 3D fix, GNSS fix OK, 12 satellites.
    payload[11] = 0x07;
    payload[20] = 3;
    payload[21] = 0x01;
    payload[23] = 12;

    // Longitude: -1.2345678°
    payload[24..28].copy_from_slice(&(-12_345_678i32).to_le_bytes());

    // Latitude: 51.2345678°
    //payload[28..32].copy_from_slice(&51_234_5678i32.to_le_bytes());
    payload[28..32].copy_from_slice(&512_345_678i32.to_le_bytes());
    // Height above ellipsoid: 123.450 m
    payload[32..36].copy_from_slice(&123_450i32.to_le_bytes());

    // Height above MSL: 100.000 m
    payload[36..40].copy_from_slice(&100_000i32.to_le_bytes());

    // Horizontal / vertical accuracy.
    payload[40..44].copy_from_slice(&500u32.to_le_bytes());
    payload[44..48].copy_from_slice(&800u32.to_le_bytes());

    // Velocity: N/E/D and ground speed, mm/s.
    payload[48..52].copy_from_slice(&1_230i32.to_le_bytes());
    payload[52..56].copy_from_slice(&(-450i32).to_le_bytes());
    payload[56..60].copy_from_slice(&120i32.to_le_bytes());
    payload[60..64].copy_from_slice(&1_310i32.to_le_bytes());

    // Heading: 12.3456°
    payload[64..68].copy_from_slice(&1_234_560u32.to_le_bytes());
    payload
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use crate::gps::{GpsData, ubx_parser::UbxParser};

    use super::*;
    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<UbxNavPvt>();
    }

    #[test]
    fn parse_nav_pvt_extracts_itow() {
        let mut payload = [0u8; UbxNavPvt::PAYLOAD_LEN];
        let itow = 123_456_789u32;
        payload[0..4].copy_from_slice(&itow.to_le_bytes());

        let result = UbxNavPvt::parse(&payload).expect("NAV-PVT should parse");

        assert_eq!(result.time_of_week_ms, itow);
    }
    #[test]
    fn parse_nav_pvt_rejects_wrong_payload_length() {
        let payload = [0u8; 91];

        assert_eq!(UbxNavPvt::parse(&payload), None);
    }
    #[test]
    fn parse_nav_pvt_extracts_status_and_accuracy() {
        let mut payload = [0u8; UbxNavPvt::PAYLOAD_LEN];

        payload[11] = 0x07;

        payload[12..16].copy_from_slice(&123_456u32.to_le_bytes());

        let nano = -123_456_789i32;
        payload[16..20].copy_from_slice(&nano.to_le_bytes());

        payload[20] = 3; // 3D fix
        payload[21] = 0x05; // example flags
        payload[22] = 0x02; // example flags2
        payload[23] = 12; // satellites

        let result = UbxNavPvt::parse(&payload).expect("NAV-PVT should parse");

        assert_eq!(result.valid, 0x07);
        assert_eq!(result.time_accuracy_ns, 123_456);
        assert_eq!(result.nano, -123_456_789);
        assert_eq!(result.fix_type, 3);
        assert_eq!(result.flags, 0x05);
        assert_eq!(result.flags2, 0x02);
        assert_eq!(result.satellite_count, 12);
    }
    #[test]
    fn parse_nav_pvt_extracts_position() {
        let mut payload = [0u8; UbxNavPvt::PAYLOAD_LEN];

        let longitude = -1_234_567i32;
        let latitude = 51_234_567i32;
        let height_ellipsoid = -12_345i32;
        let height_msl = 10_000i32;

        payload[24..28].copy_from_slice(&longitude.to_le_bytes());
        payload[28..32].copy_from_slice(&latitude.to_le_bytes());
        payload[32..36].copy_from_slice(&height_ellipsoid.to_le_bytes());
        payload[36..40].copy_from_slice(&height_msl.to_le_bytes());

        let horizontal_accuracy = 1_234u32;
        let vertical_accuracy = 2_345u32;

        payload[40..44].copy_from_slice(&horizontal_accuracy.to_le_bytes());
        payload[44..48].copy_from_slice(&vertical_accuracy.to_le_bytes());

        let result = UbxNavPvt::parse(&payload).expect("NAV-PVT should parse");

        assert_eq!(result.longitude_degrees_x1e7, longitude);
        assert_eq!(result.latitude_degrees_x1e7, latitude);
        assert_eq!(result.height_ellipsoid_mm, height_ellipsoid);
        assert_eq!(result.height_msl_mm, height_msl);
        assert_eq!(result.horizontal_accuracy_mm, horizontal_accuracy);
        assert_eq!(result.vertical_accuracy_mm, vertical_accuracy);
    }
    #[test]
    fn parse_nav_pvt_extracts_velocity_and_heading() {
        let mut payload = [0u8; UbxNavPvt::PAYLOAD_LEN];

        let velocity_north = 12_345i32;
        let velocity_east = -6_789i32;
        let velocity_down = 1_234i32;
        let ground_speed = 13_579i32;
        let heading = 123_456_789u32;

        payload[48..52].copy_from_slice(&velocity_north.to_le_bytes());
        payload[52..56].copy_from_slice(&velocity_east.to_le_bytes());
        payload[56..60].copy_from_slice(&velocity_down.to_le_bytes());
        payload[60..64].copy_from_slice(&ground_speed.to_le_bytes());
        payload[64..68].copy_from_slice(&heading.to_le_bytes());

        let result = UbxNavPvt::parse(&payload).expect("NAV-PVT should parse");

        assert_eq!(result.velocity_north_mmps, velocity_north);
        assert_eq!(result.velocity_east_mmps, velocity_east);
        assert_eq!(result.velocity_down_mmps, velocity_down);
        assert_eq!(result.ground_speed_mmps, ground_speed);
        assert_eq!(result.heading_degrees_x1e5, heading);
    }
    #[allow(unused)]
    fn test_nav_pvt() -> UbxNavPvt {
        UbxNavPvt {
            time_of_week_ms: 123_456_789,

            year: 2026,
            month: 8,
            day: 18,
            hour: 12,
            minute: 34,
            second: 56,

            valid: 0x07,
            time_accuracy_ns: 100,
            nano: 0,

            fix_type: 3,
            flags: 0x01,
            flags2: 0,
            satellite_count: 12,

            longitude_degrees_x1e7: 51_234_567,
            latitude_degrees_x1e7: -1_234_567,
            height_ellipsoid_mm: 123_450,
            height_msl_mm: 100_000,
            horizontal_accuracy_mm: 500,
            vertical_accuracy_mm: 800,

            velocity_north_mmps: 1_230,
            velocity_east_mmps: -450,
            velocity_down_mmps: 120,
            ground_speed_mmps: 1_310,
            heading_degrees_x1e5: 123_456,
        }
    }
    #[test]
    fn realistic_nav_pvt_frame_produces_gps_data() {
        let payload = make_realistic_nav_pvt_payload();
        // Build the complete UBX frame.
        let mut frame = Vec::with_capacity(8 + payload.len());

        frame.extend_from_slice(&[
            0xB5, 0x62, // sync
            0x01, 0x07, // NAV-PVT
        ]);

        #[allow(clippy::cast_possible_truncation)]
        let payload_len = (payload.len() as u16).to_le_bytes();
        frame.extend_from_slice(&payload_len);
        frame.extend_from_slice(&payload);

        // UBX Fletcher checksum covers class, ID, length and payload.
        let mut ck_a = 0u8;
        let mut ck_b = 0u8;

        for &byte in &frame[2..] {
            ck_a = ck_a.wrapping_add(byte);
            ck_b = ck_b.wrapping_add(ck_a);
        }

        frame.push(ck_a);
        frame.push(ck_b);

        let mut parser = UbxParser::new();
        let mut gps = GpsData::default();
        for byte in frame {
            if let Some(message) = parser.on_data_received(byte) {
                assert_eq!(message.class, 0x01);
                assert_eq!(message.id, 0x07);
                let nav = UbxNavPvt::parse(message.payload).expect("NAV-PVT payload should parse");
                gps.amend_with_ubx_nav_pvt(nav);
                break;
            }
        }

        assert_eq!(gps.time_of_week_ms, 45_296_789);
        assert_eq!(gps.longitude_degrees_x1e7, -12_345_678);
        assert_eq!(gps.latitude_degrees_x1e7, 512_345_678);
        assert_eq!(gps.altitude_cm, 10_000);
        assert_eq!(gps.geoid_separation_cm, 2_345);

        assert_eq!(gps.satellite_count, 12);
        assert_eq!(gps.fix, 3);
        assert_eq!(gps.is_healthy, 1);

        assert_eq!(gps.velocity_north_cmps, 123);
        assert_eq!(gps.velocity_east_cmps, -45);
        assert_eq!(gps.velocity_down_cmps, 12);
        assert_eq!(gps.ground_speed_cmps, 131);
        assert_eq!(gps.heading_deci_degrees, 1_234);
    }
}
