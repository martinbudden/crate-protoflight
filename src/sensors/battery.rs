#![allow(unused)]
#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum BatteryState {
    #[default]
    Ok,
    Warning,
    Critical,
    NotPresent,
    Init,
}

/// Per-profile battery settings (voltage thresholds, capacity).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BatteryProfile {
    /// maximum voltage per cell, used for auto-detecting battery voltage in 0.01V units, default is 430 (4.30V).
    pub max_cell_voltage_v_x100: u16,
    /// minimum voltage per cell, this triggers battery critical alarm, in 0.01V units, default is 330 (3.30V).
    pub min_cell_voltage_v_x100: u16,
    /// warning voltage per cell, this triggers battery warning alarm, in 0.01V units, default is 350 (3.50V).
    pub warning_cell_voltage_v_x100: u16,
    // Cell voltage at which the battery is deemed to be "full" 0.01V units, default is 410 (4.1V)
    pub full_cell_voltage_v_x100: u16,
    pub battery_capacity_mah: u16,
    // Number of cells in battery, used for overwriting auto-detected cell count if someone has issues with it.
    pub force_battery_cell_count: u8,
    // Percentage of remaining capacity that should trigger a battery warning
    pub consumption_warning_percentage: u8,
    pub profile_name: [u8; Self::MAX_NAME_LENGTH],
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for BatteryProfile {}

impl BatteryProfile {
    const COUNT: usize = 3;
    const MAX_NAME_LENGTH: usize = 8;
    const CELL_VOLTAGE_RANGE_MIN: u16 = 100;
    const CELL_VOLTAGE_RANGE_MAX: u16 = 500;
    const CELL_VOLTAGE_DEFAULT_MIN: u16 = 330;
    const CELL_VOLTAGE_DEFAULT_MAX: u16 = 430;
    pub const fn new() -> Self {
        Self {
            max_cell_voltage_v_x100: Self::CELL_VOLTAGE_RANGE_MAX,
            min_cell_voltage_v_x100: Self::CELL_VOLTAGE_RANGE_MIN,
            warning_cell_voltage_v_x100: 350,
            full_cell_voltage_v_x100: 410,
            battery_capacity_mah: 0,
            force_battery_cell_count: 0,
            consumption_warning_percentage: 10,
            profile_name: [0u8; Self::MAX_NAME_LENGTH],
        }
    }
}

impl Default for BatteryProfile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BatteryConfig {
    pub vbat_not_present_cell_voltage: u16, // Between vbat_max_cell_voltage and 2*this is considered to be USB powered. Below this it is not present
    pub lvc_percentage: u8,                 // Percentage of throttle when lvc is triggered

    pub voltage_meter_source: u8, // source of battery voltage meter used, either ADC or ESC
    pub current_meter_source: u8, // source of battery current meter used, either ADC, Virtual or ESC

    pub use_vbat_alerts: u8,        // Issue alerts based on VBat readings
    pub use_consumption_alerts: u8, // Issue alerts based on total power consumption
    pub vbat_hysteresis: u8,        // hysteresis for alarm in 0.01V units, default 1 = 0.01V

    pub vbat_display_lpf_period: u8, // Period of the cutoff frequency for the Vbat filter for display and startup (in 0.1 s)
    pub vbat_sag_lpf_period: u8, // Period of the cutoff frequency for the Vbat sag and PID compensation filter (in 0.1 s)
    pub ibat_lpf_period: u8,     // Period of the cutoff frequency for the Ibat filter (in 0.1 s)

    pub vbat_duration_for_warning: u8, // Period voltage has to sustain before the battery state is set to BATTERY_WARNING (in 0.1 s)
    pub vbat_duration_for_critical: u8, // Period voltage has to sustain before the battery state is set to BATTERY_CRIT (in 0.1 s)
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for BatteryConfig {}

impl BatteryConfig {
    pub const fn new() -> Self {
        Self {
            vbat_not_present_cell_voltage: 300,
            lvc_percentage: 100, // off by default at 100%
            voltage_meter_source: VoltageMeter::SOURCE_NONE,

            current_meter_source: CurrentMeter::SOURCE_NONE,

            use_vbat_alerts: 1,
            use_consumption_alerts: 0,
            vbat_hysteresis: 1, // 0.01V

            vbat_display_lpf_period: 30,
            vbat_sag_lpf_period: 2,
            ibat_lpf_period: 10,
            vbat_duration_for_warning: 0,
            vbat_duration_for_critical: 0,
        }
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoltageMeter {
    /// Voltage in 0.01V steps.
    pub display_filtered_x100: u16,
    /// Last filtered voltage sample.
    pub voltage_stable_previous_filtered_x100: u16,
    pub voltage_stable_last_update_ms: u32,
    /// Rolling bitmask, bit 1 if battery difference is within threshold, shifted left.
    pub voltage_stable_bits: u16,
    // Voltage in 0.01V steps.
    pub unfiltered_x100: u16,
    // Voltage in 0.01V steps.
    pub sag_filtered_x100: u16,
    pub low_voltage_cutoff: bool,
}

impl VoltageMeter {
    pub const SOURCE_NONE: u8 = 0;
    pub const SOURCE_ADC: u8 = 1;
    pub const SOURCE_ESC: u8 = 2;
    pub const SOURCE_COUNT: usize = 3;
    pub const SOURCE_NAMES: [&str; Self::SOURCE_COUNT] = ["NONE", "ADC", "ESC"];

    pub const fn new() -> Self {
        Self {
            display_filtered_x100: 0,
            voltage_stable_previous_filtered_x100: 0,
            voltage_stable_last_update_ms: 0,
            voltage_stable_bits: 0,
            unfiltered_x100: 0,
            sag_filtered_x100: 0,
            low_voltage_cutoff: false,
        }
    }
}

impl Default for VoltageMeter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurrentMeter {
    /// Current read by current sensor in centiampere (1/100th A).
    pub amperage_x100: i32,
    /// Current read by current sensor in centiampere (1/100th A) (unfiltered).
    pub amperage_latest_x100: i32,
    /// Milliampere hours drawn from the battery since start.
    pub mah_drawn: i32,
    /// mAh offset.
    pub mah_drawn_offset: i32,
}

impl CurrentMeter {
    pub const SOURCE_NONE: u8 = 0;
    pub const SOURCE_ADC: u8 = 1;
    pub const SOURCE_VIRTUAL: u8 = 2;
    pub const SOURCE_ESC: u8 = 3;
    pub const SOURCE_MSP: u8 = 4;
    pub const SOURCE_COUNT: usize = 5;
    pub const SOURCE_NAMES: [&str; Self::SOURCE_COUNT] = ["NONE", "ADC", "VIRTUAL", "ESC", "MSP"];
    pub const fn new() -> Self {
        Self { amperage_x100: 0, amperage_latest_x100: 0, mah_drawn: 0, mah_drawn_offset: 0 }
    }
}

impl Default for CurrentMeter {
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
        is_full::<BatteryState>();
        is_full::<BatteryConfig>();
        is_full::<BatteryProfile>();
        #[cfg(feature = "serde")]
        is_config::<BatteryConfig>();
        #[cfg(feature = "serde")]
        is_config::<BatteryProfile>();
    }
    #[test]
    fn test_new() {
        let config = BatteryConfig::new();
        assert_eq!(300, config.vbat_not_present_cell_voltage);
    }
}
