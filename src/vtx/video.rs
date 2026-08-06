#![cfg(feature = "vtx")]
#![allow(unused)]

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

use crate::vtx::VtxConfig;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum VtxType {
    #[default]
    Unsupported = 0,
    RTC6705 = 1,
    Reserved = 2,
    SmartAudio = 3,
    Tramp = 4,
    Msp = 5,
    Unknown = 0xFF,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum VtxPitMode {
    #[default]
    NA,
    Off,
    On,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum VtxBand {
    #[default]
    User = 0,
    A = 1,
    B = 2,
    E = 3,
    FatShark = 4,
    RaceBand = 5,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vtx {
    #[allow(clippy::struct_field_names)]
    vtx_type: VtxType,
    pub config: VtxConfig,
    power_value: u8,
    config_changed: bool,
    power_level_count: u8,
    pit_mode: VtxPitMode,
    pub power_index: u8,
    pub band: VtxBand,
    pub channel: u8,
    pub region: u8,
}

impl Default for Vtx {
    fn default() -> Self {
        Self::new()
    }
}

impl Vtx {
    pub fn new() -> Self {
        Self {
            vtx_type: VtxType::default(),
            config: VtxConfig::new(),
            power_value: 0,
            config_changed: false,
            power_level_count: 0,
            pit_mode: VtxPitMode::default(),
            power_index: 0,
            band: VtxBand::default(),
            channel: 0,
            region: 0,
        }
    }
}

impl Vtx {
    pub const BAND_COUNT: usize = 5;
    pub const CHANNEL_COUNT: usize = 8;
    pub const POWER_LEVEL_COUNT: usize = 8;

    const FREQUENCIES: [[u16; Vtx::CHANNEL_COUNT]; Vtx::BAND_COUNT] = [
        // Boscam A
        [5865, 5845, 5825, 5805, 5785, 5765, 5745, 5725],
        // Boscam B
        [5733, 5752, 5771, 5790, 5809, 5828, 5847, 5866],
        // Boscam E
        [5705, 5685, 5665, 5645, 5885, 5905, 5925, 5945],
        // FatShark
        [5740, 5760, 5780, 5800, 5820, 5840, 5860, 5880],
        // RaceBand
        [5658, 5695, 5732, 5769, 5806, 5843, 5880, 5917],
    ];

    const BAND_NAMES: [&str; Vtx::BAND_COUNT] = ["BOSCAM A", "BOSCAM B", "BOSCAM E", "FATSHARK", "RACEBAND"];
    const BAND_LETTERS: [char; Vtx::BAND_COUNT] = ['A', 'B', 'E', 'F', 'R'];
    const CHANNEL_NAMES: [&str; Vtx::CHANNEL_COUNT] = ["1", "2", "3", "4", "5", "6", "7", "8"];
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<VtxType>();
        is_full::<VtxPitMode>();
        is_full::<VtxBand>();
        is_full::<Vtx>();
    }
    #[test]
    fn test_new() {
        let _config = Vtx::new();
    }
}
