#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemConfig {
    pub pid_profile_index: u8,
    pub active_rate_profile: u8,
    pub task_statistics: u8,
    pub rate_profile6_pos_switch: u8,
    pub cpu_overclock: u8,
    pub power_on_arming_grace_time_seconds: u8, // in seconds
    pub board_identifier: [u8; 4],
    /// Only used for F4 and G4 targets.
    pub hse_mhz: u8,
    /// The state of the configuration (defaults / configured).
    pub configuration_state: u8,
    /// Boolean that determines whether stick arming can be used.
    pub enable_stick_arming: u8,
    active_battery_profile: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for SystemConfig {}

impl SystemConfig {
    pub const fn new() -> Self {
        Self {
            pid_profile_index: 0,
            active_rate_profile: 0,
            task_statistics: 1,
            rate_profile6_pos_switch: 0,
            cpu_overclock: 0,
            power_on_arming_grace_time_seconds: 5,
            board_identifier: [b'T', b'E', b'S', b'T'],
            hse_mhz: 0,
            configuration_state: 0,
            enable_stick_arming: 0,
            active_battery_profile: 0,
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemConfig {
    #[allow(unused)]
    pub fn active_battery_profile(&self) -> u8 {
        self.active_battery_profile
    }
    #[allow(unused)]
    pub fn set_active_battery_profile(&mut self, active_battery_profile: u8) {
        self.active_battery_profile = active_battery_profile;
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
        is_full::<SystemConfig>();
        #[cfg(feature = "serde")]
        is_config::<SystemConfig>();
    }
    #[test]
    fn test_new() {
        let _config = SystemConfig::new();
    }
}
