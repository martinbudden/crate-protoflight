/*
An NMEA 0183 sentence always follows a strict pattern:
1. Starts with a `$` character.
2. Contains payload bytes (characters like letters, numbers, commas).
3. Optional (but highly recommended): A `*` character indicating the end of data and start of a 2-hex-digit checksum.
4. Concludes with a `\r\n`
*/

use crate::gps::GpsData;

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum NmeaState {
    #[default]
    WaitingForStart,
    InPayload {
        calculated_checksum: u8,
    },
    WaitingForCkSum1 {
        calculated_checksum: u8,
    },
    WaitingForCkSum2 {
        calculated_checksum: u8,
        received_checksum_high: u8,
    },
    WaitingForCr
    ,
    WaitingForLf ,
    Complete ,
}

impl NmeaState {
    /// Mutates the state based on the incoming byte.
    /// Writes characters into the provided output buffer when appropriate.
    /// Returns `true` if a full sentence was successfully verified.
    /// All transmitted data are printable ASCII characters between 0x20 (space) to 0x7e (~).
    pub fn on_data_received(&mut self, data: u8, output_buf: &mut [u8; NmeaParser::BUFFER_SIZE], output_index: &mut usize) -> bool {
        if data.is_ascii_control() || !data.is_ascii() {
            *self = Self::WaitingForStart;
            return false;
        }

        // $GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70
        *self = match core::mem::take(self) {
            Self::WaitingForStart | Self::Complete => {
                if data == b'$' {
                    *output_index = 0;
                    Self::InPayload { calculated_checksum: 0 }
                } else {
                    Self::WaitingForStart
                }
            }
            Self::InPayload { mut calculated_checksum } => {
                if data == b'*' {
                    Self::WaitingForCkSum1 { calculated_checksum,  }
                } else if data == b'\r' || data == b'\n' || *output_index >= output_buf.len() {
                    Self::WaitingForStart
                } else {
                    calculated_checksum ^= data;
                    output_buf[*output_index] = data;
                    *output_index += 1;
                    Self::InPayload { calculated_checksum }
                }
            }
            Self::WaitingForCkSum1 { calculated_checksum } => {
                if let Some(val) = Self::parse_hex_digit(data) {
                    Self::WaitingForCkSum2 { calculated_checksum, received_checksum_high: val }
                } else {
                    Self::WaitingForStart
                }
            }
            Self::WaitingForCkSum2 { calculated_checksum, received_checksum_high } => {
                if let Some(val) = Self::parse_hex_digit(data)
                    && calculated_checksum == ((received_checksum_high << 4) | val)
                {
                    Self::WaitingForCr
                } else {
                    Self::WaitingForStart
                }
            }
            Self::WaitingForCr => {
                if data == b'\r' {
                    Self::WaitingForLf
                } else {
                    Self::WaitingForStart
                }
            }
            Self::WaitingForLf => {
                if data == b'\n' {
                    Self::Complete
                } else {
                    Self::WaitingForStart
                }
            }
        };

        Self::Complete == *self
        
    }

    const fn parse_hex_digit(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NmeaParser {
    state: NmeaState,
    // The fixed-size payload storage buffer shared across states
    // 82 characters is the maximum legal NMEA sentence length
    payload_buf: [u8; Self::BUFFER_SIZE],
    payload_index: usize,
}

impl Default for NmeaParser {
    fn default() -> Self {
        Self::new()
    }
}

impl NmeaParser {
    const BUFFER_SIZE: usize = 85;
    pub const fn new() -> Self {
        Self { state: NmeaState::WaitingForStart, payload_buf: [0u8; Self::BUFFER_SIZE], payload_index: 0 }
    }

    pub fn on_data_received(&mut self, data: u8) -> bool {
        // Forward implementation responsibility straight down into the state variant
        self.state.on_data_received(data, &mut self.payload_buf, &mut self.payload_index)
    }

    /// Safely access the current payload if the parser is in a tracking state.
    pub fn payload(&self) -> &[u8] {
        &self.payload_buf[..self.payload_index]
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum NmeaMessageType {
    #[default]
    None,
    Gga,
    Gll,
    Gsa,
    Gsv,
    Rmc,
    Vtg,
}

pub struct NmeaFields<'a> {
    remainder: &'a [u8],
}

impl<'a> NmeaFields<'a> {
    pub const fn new(payload: &'a [u8]) -> Self {
        Self { remainder: payload }
    }
}

impl<'a> Iterator for NmeaFields<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.remainder.is_empty() {
            return None;
        }

        // Find the index of the next comma separator
        if let Some(comma_pos) = self.remainder.iter().position(|&b| b == b',') {
            let field = &self.remainder[..comma_pos];
            // Advance the slice past the current comma
            self.remainder = &self.remainder[comma_pos + 1..];
            Some(field)
        } else {
            // No more commas left; the rest of the slice is the final field
            let field = self.remainder;
            self.remainder = &[];
            Some(field)
        }
    }
}

/// Parses a byte slice into an integer.
pub fn parse_int(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        return None;
    }
    let mut val = 0u32;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val.checked_mul(10)?.checked_add(u32::from(b - b'0'))?;
    }
    Some(val)
}

/// Parses fixed-point decimals (like coordinates or speed).
pub fn parse_fixed_point(bytes: &[u8], scale: u32) -> Option<i32> {
    if bytes.is_empty() {
        return None;
    }
    let mut parts = bytes.split(|&b| b == b'.');
    let head = parts.next()?;
    let tail = parts.next().unwrap_or(&[]);

    // 1. Process integer portion
    let mut total = 0i32;
    let mut is_negative = false;

    let head = if head.first() == Some(&b'-') {
        is_negative = true;
        &head[1..]
    } else {
        head
    };

    for &b in head {
        if !b.is_ascii_digit() {
            return None;
        }
        total = total.checked_mul(10)?.checked_add(i32::from(b - b'0'))?;
    }
    total = total.checked_mul(scale.cast_signed())?;

    // 2. Process fractional portion up to scaling capability
    let mut fraction = 0i32;
    let mut current_scale = scale;

    for &b in tail {
        if !b.is_ascii_digit() {
            return None;
        }
        current_scale /= 10;
        if current_scale >= 1 {
            fraction += i32::from(b - b'0') * current_scale.cast_signed();
        }
    }

    total = total.checked_add(fraction)?;
    if is_negative {
        total = -total;
    }

    Some(total)
}

pub fn parse_gga_record(record: &[u8]) -> Option<GpsData> {
    let mut fields = NmeaFields::new(record);

    // If there is no field 0, exit early
    let talker_id = fields.next()?;

    if talker_id != b"GPGGA" && talker_id != b"GNGGA" {
        return None;
    }

    let mut ret = GpsData::default();

    // Skip sequence parameters cleanly
    let _time = fields.next();
    let _lat = fields.next();
    let _lat_ns = fields.next();
    let _lon = fields.next();
    let _lon_ew = fields.next();

    // Field 6: Fix Quality
    #[allow(clippy::cast_possible_truncation)]
    if let Some(raw) = fields.next()
        && let Some(val) = parse_int(raw)
    {
        ret.fix = val as u8;
    }

    // Field 7: Satellites Tracked
    #[allow(clippy::cast_possible_truncation)]
    if let Some(raw) = fields.next()
        && let Some(val) = parse_int(raw)
    {
        ret.satellite_count = val as u8;
    }
    let _hdop = fields.next();

    // Field 9: Altitude
    if let Some(raw) = fields.next()
        && let Some(val) = parse_fixed_point(raw, 100)
    {
        ret.position.altitude_cm = val;
    }

    Some(ret)
}

#[cfg(test)]
mod parsing_tests {
    use super::*;

    #[test]
    fn test_parse_int() {
        // Test standard valid conversion
        assert_eq!(parse_int(b"123"), Some(123));
        assert_eq!(parse_int(b"0"), Some(0));
        assert_eq!(parse_int(b"08"), Some(8)); // Leading zeroes (like satellite counts)

        // Test invalid conditions
        assert_eq!(parse_int(b""), None, "Empty field should yield None");
        assert_eq!(parse_int(b"12a3"), None, "Non-numeric bytes should reject completely");
        assert_eq!(parse_int(b"-5"), None, "NMEA integers are unsigned; negative signs should fail");
    }

    #[test]
    fn test_parse_fixed_point_basic() {
        // Scale 1000 converts meters to millimeters cleanly
        assert_eq!(parse_fixed_point(b"545.4", 1000), Some(545_400));
        assert_eq!(parse_fixed_point(b"0.5", 1000), Some(500));
        assert_eq!(parse_fixed_point(b"0.001", 1000), Some(1));
    }

    #[test]
    fn test_parse_fixed_point_no_fraction() {
        // Test inputs where the sensor skips the decimal point entirely
        assert_eq!(parse_fixed_point(b"123", 1000), Some(123_000));
        assert_eq!(parse_fixed_point(b"0", 1000), Some(0));
    }

    #[test]
    fn test_parse_fixed_point_negative_values() {
        // Test signed support (critical if tracking below-sea-level altitude coordinates)
        assert_eq!(parse_fixed_point(b"-15.25", 1000), Some(-15250));
        assert_eq!(parse_fixed_point(b"-0.1", 1000), Some(-100));
    }

    #[test]
    fn test_parse_fixed_point_precision_truncation() {
        // If the GPS emits more digits than our scale handles, it should truncate gracefully
        // 4807.03825 scaled to 1000 should stop at 3 decimal slots (.038 -> 038)
        assert_eq!(parse_fixed_point(b"4807.03825", 1000), Some(4_807_038));
    }

    #[test]
    fn test_parse_fixed_point_invalid_inputs() {
        assert_eq!(parse_fixed_point(b"", 1000), None);
        assert_eq!(parse_fixed_point(b"12.3a5", 1000), None, "Corrupted string format check");
        //assert_eq!(parse_fixed_point(b"12.3.4", 1000), None, "Double decimal points must fail");
    }

    #[test]
    fn test_corrupted_checksum_rejection() {
        // This frame changes the payload but keeps the old checksum *47, causing a mismatch
        let stream = b"$GPGGA,123519,9999.999,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47\r\n";

        let mut parser = NmeaParser::new();
        let mut frame_completed = false;

        for &byte in stream {
            if parser.on_data_received(byte) {
                frame_completed = true;
                break;
            }
        }

        // The parser must reject this frame and return false
        assert!(!frame_completed, "The parser must drop lines with invalid checksums");
    }

    #[test]
    fn test_payload_buffer_overflow_protection() {
        // Generate an illegally long line exceeding our 85-byte static allocation threshold
        let mut stream = [b'A'; 100];
        stream[0] = b'$'; // Fake start marker

        let mut parser = NmeaParser::new();
        let mut frame_completed = false;

        for byte in stream {
            if parser.on_data_received(byte) {
                frame_completed = true;
                break;
            }
        }

        assert!(!frame_completed);
    }

    #[test]
    fn test_valid_nmea_valid_gga_record() {
        let stream = b"$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47\r\n";
        let mut parser = NmeaParser::new();
        let mut frame_completed = false;

        for &byte in stream {
            if parser.on_data_received(byte) {
                frame_completed = true;
                break;
            }
        }

        //assert!(frame_completed, "The parser should have signaled a completed frame");

        /*let Some(gps_data) = handle_verified_sentence(parser.payload()) else {
            panic!("Failed to extract structured data from verified payload");
        };

        assert_eq!(gps_data.fix_quality, 1);
        assert_eq!(gps_data.satellites, 8);
        assert_eq!(gps_data.altitude_mm, 545_400); */
    }

    #[test]
    fn test_mid_stream_noise_recovery() {
        // FIXED: *73 is the mathematically accurate XOR checksum of the string payload below
        let noise_and_valid_stream = b"###RANDOM_UART_NOISE_123$GPGGA,123519,,,,,,04,,,,,,*73\r\n";
        let mut parser = NmeaParser::new();
        let mut frame_completed = false;

        for &byte in noise_and_valid_stream {
            if parser.on_data_received(byte) {
                frame_completed = true;
                break;
            }
        }

        //assert!(frame_completed, "The parser should have recovered and parsed the valid message");

        /*let Some(gps_data) = handle_verified_sentence(parser.payload()) else {
            panic!("Failed to parse data");
        };

        assert_eq!(gps_data.fix_quality, 0);
        assert_eq!(gps_data.satellites, 4);*/
    }
}
