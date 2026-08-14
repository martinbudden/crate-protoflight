use crate::gps::GpsProvider;

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum NmeaParser {
    #[default]
    Init,
}

impl NmeaParser {
    pub const fn new() -> Self {
        Self::Init
    }
    // TODO: placeholder
    pub fn on_data_received(&mut self, _data: u8) -> bool {
        _ = self;
        true
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum UbloxParser {
    #[default]
    Init,
}

impl UbloxParser {
    pub const fn new() -> Self {
        Self::Init
    }
    // TODO: placeholder
    pub fn on_data_received(&mut self, _data: u8) -> bool {
        _ = self;
        true
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum GpsParser {
    Nmea(NmeaParser),
    Ublox(UbloxParser),
    #[default]
    None,
}

impl GpsParser {
    pub const fn new(gps_provider: GpsProvider) -> Option<Self> {
        match gps_provider {
            GpsProvider::Nmea => Some(GpsParser::Nmea(NmeaParser::new())),
            GpsProvider::Ublox => Some(GpsParser::Ublox(UbloxParser::new())),
            _ => None,
        }
    }

    pub fn on_data_received(&mut self, data: u8) -> bool {
        match self {
            Self::Nmea(parser) => parser.on_data_received(data),
            Self::Ublox(parser) => parser.on_data_received(data),
            Self::None => false,
        }
    }
}
