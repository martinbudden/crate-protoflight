use crate::gps::{
    GpsProvider,
    nmea_parser::NmeaParser,
    ubx_parser::{UbxMessage, UbxParser},
};

//
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GpsParserEvent<'a> {
    NmeaComplete(&'a [u8]),
    UbxMessage(UbxMessage<'a>),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum GpsParser {
    Nmea(NmeaParser),
    Ubx(UbxParser),
    #[default]
    None,
}

impl GpsParser {
    pub const fn new(gps_provider: GpsProvider) -> Option<Self> {
        match gps_provider {
            GpsProvider::Nmea => Some(GpsParser::Nmea(NmeaParser::new())),
            GpsProvider::Ubx => Some(GpsParser::Ubx(UbxParser::new())),
            _ => None,
        }
    }

    pub const fn new_unwrapped(gps_provider: GpsProvider) -> Self {
        match gps_provider {
            GpsProvider::Ubx => GpsParser::Ubx(UbxParser::new()),
            _ => GpsParser::Nmea(NmeaParser::new()),
        }
    }
}

impl GpsParser {
    pub fn on_data_received(&mut self, data: u8) -> Option<GpsParserEvent<'_>> {
        match self {
            Self::Nmea(parser) => {
                if parser.on_data_received(data) {
                    parser.payload().map(GpsParserEvent::NmeaComplete)
                } else {
                    None
                }
            }
            Self::Ubx(parser) => parser.on_data_received(data).map(GpsParserEvent::UbxMessage),
            Self::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<GpsParser>();
    }
    fn make_ubx_frame(class: u8, id: u8, payload: &[u8]) -> Vec<u8> {
        #[allow(clippy::cast_possible_truncation)]
        let payload_len = (payload.len() as u16).to_le_bytes();

        let mut frame = Vec::with_capacity(8 + payload.len());

        frame.extend_from_slice(&[
            0xB5, 0x62, // UBX sync characters
            class, id,
        ]);
        frame.extend_from_slice(&payload_len);
        frame.extend_from_slice(payload);

        let mut ck_a = 0u8;
        let mut ck_b = 0u8;

        // Checksum covers class, ID, length and payload.
        for &byte in &frame[2..] {
            ck_a = ck_a.wrapping_add(byte);
            ck_b = ck_b.wrapping_add(ck_a);
        }

        frame.extend_from_slice(&[ck_a, ck_b]);

        frame
    }
    #[test]
    fn ubx_parser_produces_ubx_message_event() {
        let payload = [0x11, 0x22, 0x33];
        let frame = make_ubx_frame(0x01, 0x02, &payload);

        let mut parser = GpsParser::Ubx(UbxParser::new());

        for byte in frame {
            if let Some(event) = parser.on_data_received(byte) {
                match event {
                    GpsParserEvent::UbxMessage(message) => {
                        assert_eq!(message.class, 0x01);
                        assert_eq!(message.id, 0x02);
                        assert_eq!(message.payload, &payload);
                    }
                    GpsParserEvent::NmeaComplete(_) => {
                        panic!("expected UBX message event");
                    }
                }

                return;
            }
        }

        panic!("expected UBX message event");
    }
    #[test]
    fn nmea_parser_produces_nmea_complete_event() {
        let record = b"$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47\r\n";

        let mut parser = GpsParser::Nmea(NmeaParser::new());

        for &byte in record {
            if let Some(event) = parser.on_data_received(byte) {
                match event {
                    GpsParserEvent::NmeaComplete(payload) => {
                        assert_eq!(payload, b"GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,");
                    }
                    GpsParserEvent::UbxMessage(_) => {
                        panic!("expected NMEA complete event");
                    }
                }

                return;
            }
        }

        panic!("expected NMEA complete event");
    }
}
