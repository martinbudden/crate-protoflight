#![cfg(feature = "osd")]

use vqm::Quaternionf32;

use crate::flight::{ArmingFlags, RxMessage};

#[cfg(feature = "battery")]
use crate::sensors::BatteryMessage;

#[derive(Debug, PartialEq)]
pub struct OsdDrawContext {
    pub orientation: Quaternionf32,
    pub arming_flags: ArmingFlags,
    pub rx_message: RxMessage,
    #[cfg(feature = "battery")]
    pub battery_message: BatteryMessage,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Osd {
    /// Timer/timestamp storage to delay or resume refreshing the canvas.
    pub resume_refresh_at_us: u32,
    // Cache mapping historical rendering durations per active element index.
    //pub element_duration_fraction_us: [u32; 32],
}

impl Default for Osd {
    fn default() -> Self {
        Self::new()
    }
}

impl Osd {
    pub const fn new() -> Self {
        Self {
            resume_refresh_at_us: 0,
            //element_duration_fraction_us: [0u32; 32],
        }
    }
}

impl Osd {
    pub const PROFILE_COUNT: usize = 2;
    pub const PROFILE_NAME_LENGTH: usize = 16;
    pub const RC_CHANNELS_COUNT: usize = 4;
    pub const RC_CHANNELS_COUNT_U8: u8 = 4;
    pub const TIMER_COUNT: usize = 2;

    pub const _LOGO_ROW_COUNT: usize = 4;
    pub const _LOGO_COLUMN_COUNT: usize = 24;

    pub const SD_ROWS: u8 = 16;
    pub const SD_COLS: u8 = 30;
    pub const _HD_ROWS: u8 = 20;
    pub const _HD_COLS: u8 = 53;

    pub const FRAMERATE_DEFAULT_HZ: u16 = 12;

    pub const ESC_RPM_ALARM_OFF: i16 = -1;
    pub const ESC_CURRENT_ALARM_OFF: i16 = -1;
    pub const ESC_TEMPERATURE_ALARM_OFF: u8 = 0;

    pub const UNITS_METRIC: u8 = 0;
    pub const _UNITS_IMPERIAL: u8 = 1;

    pub const LOGO_ARMING_OFF: u8 = 0;
    pub const _LOGO_ARMING_ON: u8 = 1;
    pub const _LOGO_ARMING_FIRST: u8 = 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<Osd>();
        //is_normal::<OsdDrawContext>();
    }
}
