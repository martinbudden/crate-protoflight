#![allow(unused)]
use serde::{Deserialize, Serialize};

pub struct BatteryState {}
impl BatteryState {
    pub const OK: u8 = 0;
    pub const WARNING: u8 = 1;
    pub const CRITICAL: u8 = 2;
    pub const NOT_PRESENT: u8 = 3;
    pub const INIT: u8 = 4;
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
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

impl BatteryConfig {
    pub const fn new() -> Self {
        Self {
            vbat_not_present_cell_voltage: 300,
            lvc_percentage: 100, // off by default at 100%
            voltage_meter_source: 0,

            current_meter_source: 0,

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

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn _is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_config<
        T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq + Serialize + for<'a> Deserialize<'a>,
    >() {
    }

    #[test]
    fn normal_types() {
        is_config::<BatteryConfig>();
    }
    #[test]
    fn test_new() {
        let config = BatteryConfig::new();
        assert_eq!(300, config.vbat_not_present_cell_voltage);
    }
}
