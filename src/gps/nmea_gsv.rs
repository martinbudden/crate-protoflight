use crate::gps::nmea_parser::{NmeaFields, parse_int};

///GSV is a little different from GGA/RMC/GSA because it describes satellites in view,
///  and a single GSV report can span multiple sentences.
/// A typical sequence looks like:
///
/// $GPGSV,3,1,11,...
/// $GPGSV,3,2,11,...
/// $GPGSV,3,3,11,...
/// | Field | Meaning                                   |
/// | ----: | ----------------------------------------- |
/// |     0 | `GPGSV`                                   |
/// |     1 | Total number of GSV messages              |
/// |     2 | Message number                            |
/// |     3 | Total satellites in view                  |
/// |   4–7 | Satellite 1: PRN, elevation, azimuth, SNR |
/// |  8–11 | Satellite 2                               |
/// | 12–15 | Satellite 3                               |
/// | 16–19 | Satellite 4                               |
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NmeaGsv {
    pub satellites_in_view: u8,
}

impl Default for NmeaGsv {
    fn default() -> Self {
        Self::new()
    }
}

impl NmeaGsv {
    pub const fn new() -> Self {
        Self { satellites_in_view: 0 }
    }
}

impl NmeaGsv {
    pub fn parse(record: &[u8]) -> Option<Self> {
        let mut fields = NmeaFields::new(record);

        let talker_id = fields.next()?;

        if talker_id.len() != 5 || &talker_id[2..] != b"GSV" {
            return None;
        }
        let mut ret = Self::default();

        // Field 1: Number of GSV messages
        let message_count = parse_int(fields.next()?)?;
        if message_count == 0 || message_count > u32::from(u8::MAX) {
            return None;
        }

        // Field 2: Message number
        let message_number = parse_int(fields.next()?)?;
        if message_number == 0 || message_number > message_count {
            return None;
        }

        // Field 3: Total satellites in view
        let satellites_in_view = parse_int(fields.next()?)?;
        ret.satellites_in_view = u8::try_from(satellites_in_view).ok()?;

        Some(ret)
    }
}
