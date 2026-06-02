#![cfg(feature = "vtx")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vtx {}

impl Vtx {
    pub const BAND_USER: u8 = 0;
    pub const BAND_A: u8 = 1;
    pub const BAND_B: u8 = 2;
    pub const BAND_E: u8 = 3;
    pub const BAND_FATSHARK: u8 = 4;
    pub const BAND_RACEBAND: u8 = 5;
    pub const BAND_COUNT: u8 = 5;
    pub const CHANNEL_COUNT: u8 = 8;
    pub const POWER_LEVEL_COUNT: u8 = 8;
}

impl Vtx {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for Vtx {
    fn default() -> Self {
        Self::new()
    }
}
