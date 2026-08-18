/*
An NMEA 0183 sentence always follows a strict pattern:
1. Starts with a `$` character.
2. Contains payload bytes (characters like letters, numbers, commas).
3. Optional (but highly recommended): A `*` character indicating the end of data and start of a 2-hex-digit checksum.
4. Concludes with a `\r\n`
*/

use crate::gps::GpsData;

#[derive(Clone, Copy, Debug, PartialEq)]
enum NmeaEvent {
    None,
    Start,
    PayloadByte(u8),
    Complete,
}

/// `NmeaState` knows about:
/// 1. $
/// 2. payload
/// 3. *
/// 4. checksum digits
/// 5. CR/LF
/// 6. checksum calculation
#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum NmeaState {
    #[default]
    WaitingForStart,
    InPayload {
        calculated_checksum: u8,
    },
    WaitingForChecksum1 {
        calculated_checksum: u8,
    },
    WaitingForChecksum2 {
        calculated_checksum: u8,
        received_checksum_high: u8,
    },
    WaitingForCr,
    WaitingForLf,
}

impl NmeaState {
    fn on_data_received(&mut self, data: u8) -> NmeaEvent {
        // '$' always starts a new NMEA sentence, regardless of the
        // current parser state.
        if data == b'$' {
            *self = Self::InPayload { calculated_checksum: 0 };

            return NmeaEvent::Start;
        }

        match *self {
            Self::WaitingForStart => {
                if data == b'$' {
                    *self = Self::InPayload { calculated_checksum: 0 };

                    NmeaEvent::Start
                } else {
                    NmeaEvent::None
                }
            }

            Self::InPayload { mut calculated_checksum } => {
                if data == b'*' {
                    *self = Self::WaitingForChecksum1 { calculated_checksum };

                    NmeaEvent::None
                } else if data.is_ascii_graphic() || data == b' ' {
                    calculated_checksum ^= data;

                    *self = Self::InPayload { calculated_checksum };

                    NmeaEvent::PayloadByte(data)
                } else {
                    *self = Self::WaitingForStart;
                    NmeaEvent::None
                }
            }

            Self::WaitingForChecksum1 { calculated_checksum } => {
                if let Some(val) = Self::parse_hex_digit(data) {
                    *self = Self::WaitingForChecksum2 { calculated_checksum, received_checksum_high: val };
                } else {
                    *self = Self::WaitingForStart;
                }

                NmeaEvent::None
            }

            Self::WaitingForChecksum2 { calculated_checksum, received_checksum_high } => {
                if let Some(val) = Self::parse_hex_digit(data) {
                    let received_checksum = (received_checksum_high << 4) | val;

                    if calculated_checksum == received_checksum {
                        *self = Self::WaitingForCr;
                    } else {
                        *self = Self::WaitingForStart;
                    }
                } else {
                    *self = Self::WaitingForStart;
                }

                NmeaEvent::None
            }

            Self::WaitingForCr => {
                if data == b'\r' {
                    *self = Self::WaitingForLf;
                } else {
                    *self = Self::WaitingForStart;
                }

                NmeaEvent::None
            }

            Self::WaitingForLf => {
                if data == b'\n' {
                    *self = Self::WaitingForStart;
                    NmeaEvent::Complete
                } else {
                    *self = Self::WaitingForStart;
                    NmeaEvent::None
                }
            }
        }
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

/// `NmeaParser` knows about:
/// 1. the payload buffer
/// 2. the payload index
/// 3. buffer overflow
/// 4. whether a complete sentence is available.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NmeaParser {
    state: NmeaState,
    payload_buf: [u8; Self::BUFFER_SIZE],
    payload_index: usize,
    complete: bool,
}

impl Default for NmeaParser {
    fn default() -> Self {
        Self::new()
    }
}

impl NmeaParser {
    // Maximum NMEA sentence length is 82 bytes including '$' and CR/LF.
    // This buffer contains only the payload between '$' and '*'.
    // `BUFFER_SIZE` is rounded up to 80 which is larger than the payload permitted by the NMEA 0183 sentence length limit.
    const BUFFER_SIZE: usize = 80;

    pub const fn new() -> Self {
        Self {
            state: NmeaState::WaitingForStart,
            payload_buf: [0u8; Self::BUFFER_SIZE],
            payload_index: 0,
            complete: false,
        }
    }

    pub fn on_data_received(&mut self, data: u8) -> bool {
        // A new byte means that the previously completed sentence
        // has been consumed/expired.
        self.complete = false;

        match self.state.on_data_received(data) {
            NmeaEvent::None => {}

            NmeaEvent::Start => {
                self.payload_index = 0;
            }
            NmeaEvent::PayloadByte(byte) => {
                if self.payload_index >= self.payload_buf.len() {
                    // Payload is too long. Discard the sentence.
                    self.state = NmeaState::WaitingForStart;
                    self.payload_index = 0;
                } else {
                    self.payload_buf[self.payload_index] = byte;
                    self.payload_index += 1;
                }
            }
            NmeaEvent::Complete => {
                self.complete = true;
            }
        }
        self.complete
    }

    pub fn payload(&self) -> Option<&[u8]> {
        if self.complete { Some(&self.payload_buf[..self.payload_index]) } else { None }
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

/// Parses a decimal fixed-point value without allocation.
///
/// `scale` specifies the number of units per whole number, and must be a
/// positive power of ten. For example:
///
/// - `scale = 10`   -> one decimal place
/// - `scale = 100`  -> two decimal places
/// - `scale = 1000` -> three decimal places
///
/// Excess fractional digits are truncated.
///
/// Examples:
/// - `12.34` with scale 100 -> 1234
/// - `12.3`  with scale 100 -> 1230
/// - `12`    with scale 100 -> 1200
pub fn parse_fixed_point(bytes: &[u8], scale: u32) -> Option<i32> {
    if bytes.is_empty() || scale == 0 {
        return None;
    }

    let mut index = 0;
    let negative = bytes[0] == b'-';

    if negative {
        index = 1;

        // "-" by itself isn't a number.
        if index == bytes.len() {
            return None;
        }
    }

    let mut integer_part = 0i32;
    let mut saw_integer_digit = false;

    // Parse integer part.
    while index < bytes.len() {
        let byte = bytes[index];

        if byte == b'.' {
            break;
        }

        let digit = byte.checked_sub(b'0')?;
        if digit > 9 {
            return None;
        }

        saw_integer_digit = true;

        integer_part = integer_part.checked_mul(10)?.checked_add(i32::from(digit))?;

        index += 1;
    }

    // Don't accept ".5".
    if !saw_integer_digit {
        return None;
    }

    let scale_i32 = i32::try_from(scale).ok()?;

    let mut result = integer_part.checked_mul(scale_i32)?;

    // Parse fractional part, if present.
    if index < bytes.len() {
        // Skip '.'.
        index += 1;

        let mut fractional_scale = scale;

        while index < bytes.len() {
            let byte = bytes[index];

            let digit = byte.checked_sub(b'0')?;
            if digit > 9 {
                return None;
            }

            // Once the requested precision has been reached, continue
            // validating digits but stop adding them to the result.
            fractional_scale /= 10;

            if fractional_scale != 0 {
                let fractional_scale_i32 = i32::try_from(fractional_scale).ok()?;
                let contribution = i32::from(digit).checked_mul(fractional_scale_i32)?;

                result = result.checked_add(contribution)?;
            }

            index += 1;
        }
    }

    if negative { result.checked_neg() } else { Some(result) }
}
/// Parses an NMEA latitude/longitude coordinate.
///
/// NMEA coordinates are represented as:
///
/// - latitude:  `ddmm.mmmm`
/// - longitude: `dddmm.mmmm`
///
/// `direction` must be `N`, `S`, `E`, or `W`.
///
/// The result is returned as degrees × 1e7, rounded to the nearest unit.
pub fn parse_nmea_coordinate(value: &[u8], direction: u8) -> Option<i32> {
    if value.is_empty() {
        return None;
    }

    let negative = match direction {
        b'N' | b'E' => false,
        b'S' | b'W' => true,
        _ => return None,
    };

    // Find the decimal point separating degrees/minutes from the
    // fractional part.
    let decimal_pos = value.iter().position(|&b| b == b'.')?;

    // There must be either:
    //   ddmm.mmmm   -> 4 digits before '.'
    //   dddmm.mmmm  -> 5 digits before '.'
    if decimal_pos != 4 && decimal_pos != 5 {
        return None;
    }

    // The two digits immediately before the decimal point are the
    // integer portion of the minutes.
    let minute_start = decimal_pos - 2;

    // Parse degrees.
    let mut degrees = 0u32;

    for &byte in &value[..minute_start] {
        let digit = byte.checked_sub(b'0')?;

        if digit > 9 {
            return None;
        }

        degrees = degrees.checked_mul(10)?.checked_add(u32::from(digit))?;
    }

    // Parse integer minutes.
    let minute_tens = value[minute_start].checked_sub(b'0')?;

    let minute_units = value[minute_start + 1].checked_sub(b'0')?;

    if minute_tens > 9 || minute_units > 9 {
        return None;
    }

    let minutes = u32::from(minute_tens) * 10 + u32::from(minute_units);

    // Minutes must be in [0, 60).
    if minutes >= 60 {
        return None;
    }

    // Parse fractional minutes, normalizing to 1/10000 minute.
    //
    // NMEA normally provides four decimal places, but accepting fewer
    // is harmless. Additional digits are deliberately truncated.
    let mut fractional = 0u32;
    let mut fractional_digits = 0;

    for &byte in &value[decimal_pos + 1..] {
        let digit = byte.checked_sub(b'0')?;

        if digit > 9 {
            return None;
        }

        if fractional_digits < 4 {
            fractional = fractional * 10 + u32::from(digit);
            fractional_digits += 1;
        }
    }

    // Scale fractional minutes to 1/10000 minute.
    while fractional_digits < 4 {
        fractional *= 10;
        fractional_digits += 1;
    }

    let minutes_x1e4 = minutes.checked_mul(10_000)?.checked_add(fractional)?;

    // Convert minutes to degrees × 1e7:
    //
    // minutes_x1e4 * 1e7 / (60 * 1e4)
    // = minutes_x1e4 * 500 / 3
    //
    // Add 1 before division to round to nearest integer.
    let minute_degrees_x1e7 = minutes_x1e4.checked_mul(50)?.checked_add(1)? / 3;

    let result = degrees.checked_mul(10_000_000)?.checked_add(minute_degrees_x1e7)?;

    let result = i32::try_from(result).ok()?;

    if negative { result.checked_neg() } else { Some(result) }
}

/// Parses an NMEA UTC time (`hhmmss` or `hhmmss.ss`) into milliseconds
/// since midnight.
///
/// Fractional seconds beyond millisecond precision are truncated.
pub fn parse_nmea_time(bytes: &[u8]) -> Option<u32> {
    if bytes.len() < 6 {
        return None;
    }
    // hhmmss
    let hour = parse_two_digits(&bytes[0..2])?;
    let minute = parse_two_digits(&bytes[2..4])?;
    let second = parse_two_digits(&bytes[4..6])?;
    if hour >= 24 || minute >= 60 || second >= 60 {
        return None;
    }
    let mut milliseconds = u32::from(hour) * 3_600_000 + u32::from(minute) * 60_000 + u32::from(second) * 1_000;

    // Optional fractional seconds.
    if bytes.len() > 6 {
        if bytes[6] != b'.' {
            return None;
        }
        let mut fraction_ms = 0u32;
        for (index, &byte) in bytes[7..].iter().enumerate() {
            let digit = byte.checked_sub(b'0')?;
            if digit > 9 {
                return None;
            }
            match index {
                0 => fraction_ms += u32::from(digit) * 100,
                1 => fraction_ms += u32::from(digit) * 10,
                2 => fraction_ms += u32::from(digit),
                _ => {}
            }
        }
        milliseconds = milliseconds.checked_add(fraction_ms)?;
    }
    Some(milliseconds)
}

fn parse_two_digits(bytes: &[u8]) -> Option<u8> {
    if bytes.len() != 2 {
        return None;
    }

    let tens = bytes[0].checked_sub(b'0')?;
    let units = bytes[1].checked_sub(b'0')?;

    if tens > 9 || units > 9 {
        return None;
    }

    Some(tens * 10 + units)
}
pub struct NmeaFields<'a> {
    remainder: Option<&'a [u8]>,
}

impl<'a> NmeaFields<'a> {
    pub const fn new(payload: &'a [u8]) -> Self {
        Self { remainder: Some(payload) }
    }
}

impl<'a> Iterator for NmeaFields<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let remainder = self.remainder.take()?;

        if let Some(comma_pos) = remainder.iter().position(|&b| b == b',') {
            let field = &remainder[..comma_pos];
            self.remainder = Some(&remainder[comma_pos + 1..]);
            Some(field)
        } else {
            // This is the final field. Setting remainder to None means
            // that even an empty final field is returned once.
            Some(remainder)
        }
    }
}

// TODO: add parsing of RMC, GSVm and GSA frames.

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
pub fn parse_gga_record(record: &[u8]) -> Option<GpsData> {
    let mut fields = NmeaFields::new(record);

    let talker_id = fields.next()?;

    if talker_id.len() != 5 || &talker_id[2..] != b"GGA" {
        return None;
    }

    let mut ret = GpsData::default();

    // Field 1: UTC time
    let time = fields.next()?;
    ret.time_of_day_ms = parse_nmea_time(time)?;

    // Field 2/3: latitude and N/S
    let latitude = fields.next()?;
    let latitude_direction = fields.next()?;

    // Field 4/5: longitude and E/W
    let longitude = fields.next()?;
    let longitude_direction = fields.next()?;

    if latitude_direction.len() != 1 || longitude_direction.len() != 1 {
        return None;
    }

    ret.position.latitude_degrees_x1e7 = parse_nmea_coordinate(latitude, latitude_direction[0])?;

    ret.position.longitude_degrees_x1e7 = parse_nmea_coordinate(longitude, longitude_direction[0])?;

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
    ret.dilution_of_precision_positional = i16::try_from(hdop).ok()?;

    // Field 9/10: Altitude and units
    let altitude = fields.next()?;
    let altitude_units = fields.next()?;
    if altitude_units != b"M" {
        return None;
    }
    ret.position.altitude_cm = parse_fixed_point(altitude, 100)?;

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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_sentence_is_accepted() {
        let sentence = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70\r\n";
        let mut parser = NmeaParser::new();
        let mut complete = false;
        for &byte in sentence {
            complete = parser.on_data_received(byte);
        }
        assert!(complete);
        assert_eq!(
            parser.payload(),
            Some(b"GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30".as_slice())
        );
    }

    #[test]
    fn invalid_checksum_is_rejected() {
        let sentence = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*71\r\n";
        let mut parser = NmeaParser::new();
        for &byte in sentence {
            assert!(!parser.on_data_received(byte));
        }
        assert_eq!(parser.payload(), None);
    }

    #[test]
    fn invalid_checksum_character_is_rejected() {
        let sentence = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*7G\r\n";
        let mut parser = NmeaParser::new();
        for &byte in sentence {
            assert!(!parser.on_data_received(byte));
        }
        assert_eq!(parser.payload(), None);
    }
    #[test]
    fn missing_lf_is_rejected() {
        let sentence = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70\r";
        let mut parser = NmeaParser::new();
        for &byte in sentence {
            assert!(!parser.on_data_received(byte));
        }
        assert_eq!(parser.payload(), None);
    }

    #[test]
    fn missing_cr_is_rejected() {
        let sentence = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70\n";
        let mut parser = NmeaParser::new();
        for &byte in sentence {
            assert!(!parser.on_data_received(byte));
        }
        assert_eq!(parser.payload(), None);
    }

    #[test]
    fn back_to_back_sentences_are_parsed() {
        let sentence1 = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70\r\n";
        let sentence2 = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70\r\n";
        let mut parser = NmeaParser::new();
        let mut complete_count = 0;
        for &byte in sentence1.iter().chain(sentence2) {
            if parser.on_data_received(byte) {
                complete_count += 1;
                assert_eq!(
                    parser.payload(),
                    Some(b"GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30".as_slice())
                );
            }
        }
        assert_eq!(complete_count, 2);
    }

    #[test]
    fn garbage_before_sentence_is_ignored() {
        let garbage = b"hello123";
        let sentence = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70\r\n";
        let mut parser = NmeaParser::new();
        for &byte in garbage.iter().chain(sentence) {
            _ = parser.on_data_received(byte);
        }
        assert_eq!(
            parser.payload(),
            Some(b"GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30".as_slice())
        );
    }

    #[test]
    fn recovers_after_invalid_sentence() {
        let invalid = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*71\r\n";
        let valid = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70\r\n";
        let mut parser = NmeaParser::new();
        for &byte in invalid.iter().chain(valid) {
            _ = parser.on_data_received(byte);
        }
        assert_eq!(
            parser.payload(),
            Some(b"GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30".as_slice())
        );
    }

    #[test]
    fn resynchronises_on_new_dollar() {
        let valid = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70\r\n";

        // The first sentence is corrupt/incomplete. A new '$' appears
        // before the corrupt sentence has terminated.
        let corrupt_prefix = b"$GPGSV,3,1,11,10,63,137,17,07$";

        let mut parser = NmeaParser::new();

        let mut complete_count = 0;

        for &byte in corrupt_prefix.iter().chain(valid) {
            if parser.on_data_received(byte) {
                complete_count += 1;
            }
        }

        assert_eq!(complete_count, 1);

        assert_eq!(
            parser.payload(),
            Some(b"GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30".as_slice())
        );
    }
    #[test]
    fn payload_overflow_is_rejected() {
        let mut parser = NmeaParser::new();

        // Start a sentence.
        assert!(!parser.on_data_received(b'$'));

        // Fill the payload buffer exactly.
        for _ in 0..NmeaParser::BUFFER_SIZE {
            assert!(!parser.on_data_received(b'A'));
        }

        // One more payload byte must cause the sentence to be rejected.
        assert!(!parser.on_data_received(b'B'));

        assert_eq!(parser.payload(), None);
    }
    #[test]
    fn recovers_after_payload_overflow() {
        let valid = b"$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70\r\n";

        let mut parser = NmeaParser::new();

        // Start a sentence and deliberately overflow the payload buffer.
        assert!(!parser.on_data_received(b'$'));

        for _ in 0..NmeaParser::BUFFER_SIZE {
            assert!(!parser.on_data_received(b'A'));
        }

        assert!(!parser.on_data_received(b'B'));

        // A new '$' should allow the parser to recover.
        let mut complete = false;

        for &byte in valid {
            complete = parser.on_data_received(byte);
        }

        assert!(complete);

        assert_eq!(
            parser.payload(),
            Some(b"GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30".as_slice())
        );
    }

    #[test]
    fn fixed_point_basic() {
        assert_eq!(parse_fixed_point(b"12.34", 100), Some(1234));
        assert_eq!(parse_fixed_point(b"12.3", 100), Some(1230));
        assert_eq!(parse_fixed_point(b"12", 100), Some(1200));
    }
    #[test]
    fn fixed_point_negative() {
        assert_eq!(parse_fixed_point(b"-12.34", 100), Some(-1234));
        assert_eq!(parse_fixed_point(b"-12.3", 100), Some(-1230));
    }
    #[test]
    fn fixed_point_truncates_extra_precision() {
        assert_eq!(parse_fixed_point(b"12.345", 100), Some(1234));
    }
    #[test]
    fn fixed_point_rejects_invalid_input() {
        assert_eq!(parse_fixed_point(b"", 100), None);
        assert_eq!(parse_fixed_point(b"-", 100), None);
        assert_eq!(parse_fixed_point(b"12.x", 100), None);
        assert_eq!(parse_fixed_point(b"12.3.4", 100), None);
        assert_eq!(parse_fixed_point(b"abc", 100), None);
    }
    #[test]
    fn fixed_point_rejects_overflow() {
        assert_eq!(parse_fixed_point(b"999999999999999999", 100), None);
    }

    #[test]
    fn parse_nmea_coordinate_north() {
        assert_eq!(parse_nmea_coordinate(b"4916.45", b'N'), Some(492_741_667));
    }

    #[test]
    fn parse_nmea_coordinate_south() {
        assert_eq!(parse_nmea_coordinate(b"4916.45", b'S'), Some(-492_741_667));
    }
    #[test]
    fn parse_nmea_coordinate_east() {
        assert_eq!(parse_nmea_coordinate(b"12311.12", b'E'), Some(1_231_853_333));
    }

    #[test]
    fn parse_nmea_coordinate_west() {
        assert_eq!(parse_nmea_coordinate(b"12311.12", b'W'), Some(-1_231_853_333));
    }
    #[test]
    fn parse_nmea_coordinate_rejects_invalid_direction() {
        assert_eq!(parse_nmea_coordinate(b"4916.45", b'X'), None);
    }

    #[test]
    fn parse_nmea_coordinate_rejects_invalid_minutes() {
        assert_eq!(parse_nmea_coordinate(b"4960.00", b'N'), None);
    }

    #[test]
    fn parse_nmea_coordinate_rejects_missing_decimal() {
        assert_eq!(parse_nmea_coordinate(b"491645", b'N'), None);
    }

    #[test]
    fn parse_nmea_coordinate_rejects_bad_digit() {
        assert_eq!(parse_nmea_coordinate(b"49x6.45", b'N'), None);
    }

    #[test]
    fn parse_nmea_coordinate_rejects_empty() {
        assert_eq!(parse_nmea_coordinate(b"", b'N'), None);
    }
    #[test]
    fn parse_nmea_coordinate_zero() {
        assert_eq!(parse_nmea_coordinate(b"0000.00", b'N'), Some(0));
    }

    #[test]
    fn parse_nmea_coordinate_exact_degree() {
        assert_eq!(parse_nmea_coordinate(b"4900.00", b'N'), Some(490_000_000));
    }

    #[test]
    fn parse_nmea_coordinate_near_sixty_minutes() {
        assert_eq!(parse_nmea_coordinate(b"4959.9999", b'N'), Some(499_999_983));
    }

    #[test]
    fn parse_gga_record_extracts_position_and_fix() {
        let record = b"GPGGA,123519.500,4916.45,N,12311.12,W,1,08,0.9,545.4,M,46.9,M,,";

        #[allow(clippy::expect_used)]
        let result = parse_gga_record(record).expect("GGA record should parse");

        assert_eq!(result.time_of_day_ms, 45_319_500);

        assert_eq!(result.position.latitude_degrees_x1e7, 492_741_667);

        assert_eq!(result.position.longitude_degrees_x1e7, -1_231_853_333);

        assert_eq!(result.position.altitude_cm, 54_540);

        assert_eq!(result.dilution_of_precision_positional, 9);

        assert_eq!(result.fix, 1);
        assert_eq!(result.satellite_count, 8);
        assert_eq!(result.geoid_separation_cm, 4_690);
    }
    #[test]
    fn parse_gga_record_rejects_invalid_latitude() {
        let record = b"GPGGA,123519,4916.X,N,12311.12,W,1,08,0.9,545.4,M,46.9,M,,";

        assert_eq!(parse_gga_record(record), None);
    }
    #[test]
    fn parse_gga_record_rejects_invalid_fix() {
        let record = b"GPGGA,123519,4916.45,N,12311.12,W,X,08,0.9,545.4,M,46.9,M,,";

        assert_eq!(parse_gga_record(record), None);
    }
    #[test]
    fn parse_gga_record_handles_negative_geoid_separation() {
        let record = b"GPGGA,123519.500,4916.45,N,12311.12,W,1,08,0.9,545.4,M,-46.9,M,,";

        let result = parse_gga_record(record).expect("GGA record should parse");

        assert_eq!(result.position.altitude_cm, 54_540);
        assert_eq!(result.geoid_separation_cm, -4_690);
    }
    #[test]
    fn parse_nmea_time_basic() {
        assert_eq!(parse_nmea_time(b"123519"), Some(45_319_000));
    }

    #[test]
    fn parse_nmea_time_with_fraction() {
        assert_eq!(parse_nmea_time(b"123519.500"), Some(45_319_500));
    }

    #[test]
    fn parse_nmea_time_truncates_excess_precision() {
        assert_eq!(parse_nmea_time(b"123519.5009"), Some(45_319_500));
    }

    #[test]
    fn parse_nmea_time_midnight() {
        assert_eq!(parse_nmea_time(b"000000"), Some(0));
    }

    #[test]
    fn parse_nmea_time_end_of_day() {
        assert_eq!(parse_nmea_time(b"235959.999"), Some(86_399_999));
    }
    #[test]
    fn parse_nmea_time_rejects_invalid_hour() {
        assert_eq!(parse_nmea_time(b"240000"), None);
    }

    #[test]
    fn parse_nmea_time_rejects_invalid_minute() {
        assert_eq!(parse_nmea_time(b"126000"), None);
    }

    #[test]
    fn parse_nmea_time_rejects_invalid_second() {
        assert_eq!(parse_nmea_time(b"125960"), None);
    }

    #[test]
    fn parse_nmea_time_rejects_bad_digit() {
        assert_eq!(parse_nmea_time(b"1235X9"), None);
    }

    #[test]
    fn parse_nmea_time_rejects_bad_separator() {
        assert_eq!(parse_nmea_time(b"123519X500"), None);
    }
}
