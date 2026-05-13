#![allow(unused)]
pub struct OsdSymbols {}

impl OsdSymbols {
    // Artificial horizon center screen Graphics
    pub const AH_CENTER_LINE: u8 = 0x72;
    pub const AH_CENTER: u8 = 0x73;
    pub const AH_CENTER_LINE_RIGHT: u8 = 0x74;
    pub const AH_RIGHT: u8 = 0x02;
    pub const AH_LEFT: u8 = 0x03;
    pub const AH_DECORATION: u8 = 0x13;
    // Artificial horizon bars
    pub const AH_BAR9_0: u8 = 0x80;

    //Misc
    pub const NONE: u8 = 0x00;
    pub const END_OF_FONT: u8 = 0xFF;
    pub const BLANK: u8 = 0x20;
    pub const HYPHEN: u8 = 0x2D;
    pub const BLACKBOX_LOG: u8 = 0x10;
    pub const HOME_FLAG: u8 = 0x11;

    // GPS and navigation
    pub const LAT: u8 = 0x89;
    pub const LON: u8 = 0x98;
    pub const ALTITUDE: u8 = 0x7F;
    pub const TOTAL_DISTANCE: u8 = 0x71;
    pub const OVER_HOME: u8 = 0x05;

    // RSSI
    pub const RSSI: u8 = 0x01;
    pub const LINK_QUALITY: u8 = 0x7B;

    // Throttle Position (%)
    pub const THR: u8 = 0x04;

    // Unit Icons (Metric)
    pub const M: u8 = 0x0C;
    pub const KM: u8 = 0x7D;
    pub const C: u8 = 0x0E;

    // Unit Icons (Imperial)
    pub const FT: u8 = 0x0F;
    pub const MILES: u8 = 0x7E;
    pub const F: u8 = 0x0D;

    // Heading Graphics
    pub const HEADING_N: u8 = 0x18;
    pub const HEADING_S: u8 = 0x19;
    pub const HEADING_E: u8 = 0x1A;
    pub const HEADING_W: u8 = 0x1B;
    pub const HEADING_DIVIDED_LINE: u8 = 0x1C;
    pub const HEADING_LINE: u8 = 0x1D;

    // Satellite Graphics
    pub const SAT_L: u8 = 0x1E;
    pub const SAT_R: u8 = 0x1F;

    // Direction arrows
    pub const ARROW_SOUTH: u8 = 0x60;
    pub const ARROW_2: u8 = 0x61;
    pub const ARROW_3: u8 = 0x62;
    pub const ARROW_4: u8 = 0x63;
    pub const ARROW_EAST: u8 = 0x64;
    pub const ARROW_6: u8 = 0x65;
    pub const ARROW_7: u8 = 0x66;
    pub const ARROW_8: u8 = 0x67;
    pub const ARROW_NORTH: u8 = 0x68;
    pub const ARROW_10: u8 = 0x69;
    pub const ARROW_11: u8 = 0x6A;
    pub const ARROW_12: u8 = 0x6B;
    pub const ARROW_WEST: u8 = 0x6C;
    pub const ARROW_14: u8 = 0x6D;
    pub const ARROW_15: u8 = 0x6E;
    pub const ARROW_16: u8 = 0x6F;

    pub const ARROW_SMALL_UP: u8 = 0x75;
    pub const ARROW_SMALL_DOWN: u8 = 0x76;

    // Time
    pub const ON_M: u8 = 0x9B;
    pub const FLY_M: u8 = 0x9C;

    // Lap Timer
    pub const CHECKERED_FLAG: u8 = 0x24;
    pub const PREV_LAP_TIME: u8 = 0x79;

    // Speed
    pub const SPEED: u8 = 0x70;
    pub const KPH: u8 = 0x9E;
    pub const MPH: u8 = 0x9D;
    pub const MPS: u8 = 0x9F;
    pub const FTPS: u8 = 0x99;

    // Menu cursor
    pub const CURSOR: u8 = Self::AH_LEFT;

    // _stick overlays
    pub const STICK_OVERLAY_SPRITE_HIGH: u8 = 0x08;
    pub const STICK_OVERLAY_SPRITE_MID: u8 = 0x09;
    pub const STICK_OVERLAY_SPRITE_LOW: u8 = 0x0A;
    pub const STICK_OVERLAY_CENTER: u8 = 0x0B;
    pub const STICK_OVERLAY_VERTICAL: u8 = 0x16;
    pub const STICK_OVERLAY_HORIZONTAL: u8 = 0x17;
}
