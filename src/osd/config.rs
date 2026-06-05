#![cfg(feature = "osd")]

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

use crate::{
    display::{DisplayPortBackground, DisplayPortDeviceType},
    osd::{Osd, elements::OsdElements},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OsdConfig {
    pub profile: [[u8; Osd::PROFILE_COUNT]; Osd::PROFILE_NAME_LENGTH + 2], // extra byte for zero terminator and extra byte to even-align
    pub rc_channels: [i8; Osd::RC_CHANNELS_COUNT],                         // RC channel values to display, -1 if none
    pub timers: [u16; Osd::TIMER_COUNT],

    pub enabled_warnings_flags: u32,
    pub enabled_stats_flags: u32,

    pub framerate_hz: u16,

    pub cap_alarm: u16,
    pub alt_alarm: u16,
    pub link_quality_alarm: u16,
    pub rssi_dbm_alarm: i16,
    pub rsnr_alarm: i16,
    pub distance_alarm: u16,
    pub esc_rpm_alarm: i16,
    pub esc_current_alarm: i16,
    pub esc_temperature_alarm: u8,
    pub core_temperature_alarm: u8,
    pub rssi_alarm: u8,

    pub units: u8,

    pub aux_scale: u16,
    pub aux_channel: u8,
    pub aux_symbol: u8,

    pub logo_on_arming: u8,
    pub logo_on_arming_duration: u8, // display duration in 0.1s units
    pub arming_logo_attribute: u8,   // font attribute to use to display the logo on arming

    pub ah_max_pitch: u8,
    pub ah_max_roll: u8,
    pub ah_invert: u8,
    pub osd_profile_index: u8,
    pub overlay_radio_mode: u8,
    pub gps_sats_show_pdop: u8,
    pub camera_frame_width: u8,
    pub camera_frame_height: u8,
    pub cms_background_type: u8, // whether the CMS background is transparent or opaque
    pub stats_show_cell_value: u8,
    pub osd_craft_name_messages: u8, // Insert LQ/RSSI-dBm and warnings into CraftName
    pub display_port_device_type: u8,
    pub canvas_column_count: u8,
    pub canvas_row_count: u8,
    pub osd_use_quick_menu: u8,
    pub osd_show_spec_prearm: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for OsdConfig {}

impl OsdConfig {
    pub const fn new() -> Self {
        Self {
            profile: [[0u8; Osd::PROFILE_COUNT]; Osd::PROFILE_NAME_LENGTH + 2], // extra byte for zero terminator and extra byte to even-align
            rc_channels: [0i8; Osd::RC_CHANNELS_COUNT],
            timers: [0u16; Osd::TIMER_COUNT],
            enabled_warnings_flags: 0,
            enabled_stats_flags: 0,

            framerate_hz: Osd::FRAMERATE_DEFAULT_HZ,
            cap_alarm: 2200,
            alt_alarm: 100, // meters or feet depend on configuration
            link_quality_alarm: 80,
            rssi_dbm_alarm: -60,
            rsnr_alarm: 4,
            distance_alarm: 0,
            esc_rpm_alarm: Osd::ESC_RPM_ALARM_OFF,
            esc_current_alarm: Osd::ESC_CURRENT_ALARM_OFF,
            esc_temperature_alarm: Osd::ESC_TEMPERATURE_ALARM_OFF,
            core_temperature_alarm: 70, // a temperature above 70C should produce a warning, lockups have been reported above 80C
            rssi_alarm: 20,

            units: Osd::UNITS_METRIC,

            aux_scale: 200,
            aux_channel: 1,
            aux_symbol: b'A',

            logo_on_arming: Osd::LOGO_ARMING_OFF,
            logo_on_arming_duration: 5, // 05 seconds
            arming_logo_attribute: 0,

            ah_max_pitch: 20, // 20 degrees
            ah_max_roll: 40,  // 40 degrees
            ah_invert: 0,

            osd_profile_index: 0,
            overlay_radio_mode: 2,
            gps_sats_show_pdop: 0,

            camera_frame_width: 24,
            camera_frame_height: 11,

            cms_background_type: DisplayPortBackground::Transparent as u8,
            stats_show_cell_value: 0,
            osd_craft_name_messages: 0, // Insert LQ/RSSI-dBm and warnings into CraftName
            // Make it obvious on the configurator that the FC doesn't support HD
            display_port_device_type: DisplayPortDeviceType::Auto as u8,
            canvas_column_count: Osd::SD_COLS,
            canvas_row_count: Osd::SD_ROWS,
            osd_use_quick_menu: 0,
            osd_show_spec_prearm: 0,
        }
    }
}

impl Default for OsdConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OsdStatsConfig {
    pub total_flights: u32,
    pub total_time_s: u32,
    pub total_distance_m: u32,
    pub mah_used: u32,
    pub min_armed_time_s: i8,
    pub save_move_limit: u8, // gyro rate limit for saving stats upon disarm
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for OsdStatsConfig {}

impl OsdStatsConfig {
    pub const fn new() -> Self {
        Self {
            total_flights: 0,
            total_time_s: 0,
            total_distance_m: 0,
            mah_used: 0,
            min_armed_time_s: 0,
            save_move_limit: 0, // gyro rate limit for saving stats upon disarm
        }
    }
}

impl Default for OsdStatsConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Osd Elements configuration array: 2 bits for type, 2 bits for profile, 6 bits for y, 6 bits for x.
pub struct OsdElementsConfig {
    pub positions: [u16; OsdElements::COUNT],
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for OsdElementsConfig {}

impl OsdElementsConfig {
    pub const fn new() -> Self {
        Self { positions: [0u16; OsdElements::COUNT] }
    }
}

impl Default for OsdElementsConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
#[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<OsdConfig>();
#[cfg(feature = "serde")]
        is_config::<OsdConfig>();
        is_full::<OsdStatsConfig>();
#[cfg(feature = "serde")]
        is_config::<OsdStatsConfig>();
        is_full::<OsdElementsConfig>();
#[cfg(feature = "serde")]
        is_config::<OsdElementsConfig>();
    }
    #[test]
    fn test_new() {
        let config = OsdConfig::new();
        assert_eq!(2200, config.cap_alarm);
    }
}
