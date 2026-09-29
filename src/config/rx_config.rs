pub use radio_controllers::RadioType;

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct RxConfig {
    pub serial_rx_provider: RadioType,
    pub serial_rx_inverted: u8, // invert the serial RX protocol compared to its default setting.
    pub half_duplex: u8,        // allow rx to operate in half duplex mode on STM32 F4, ignored for F1 and F3.
    pub rssi_channel: u8,
    pub rssi_scale: u8,
    pub rssi_invert: u8,
    pub rssi_offset: i8,                 // offset applied to the RSSI value
    pub fpv_cam_angle_degrees: u8,       // Camera angle to be scaled into rc commands
    pub air_mode_activate_threshold: u8, // Throttle setpoint percent where airmode gets activated
    pub spektrum_sat_bind: u8,           // number of bind pulses for Spektrum satellite radios
    pub mid_rc: u16, // Some radios don't have a neutral point centered on 1500. This can be changed here.
    pub min_check: u16, // minimum rc
    pub max_check: u16, // maximum rc
    pub rx_min_us: u16, // rx_min in microseconds
    pub rx_max_us: u16, // rx_max in microseconds
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for RxConfig {}

impl Default for RxConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RxConfig {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            serial_rx_provider: RadioType::Crsf,
            serial_rx_inverted: 0,
            half_duplex: 0,
            rssi_channel: 0,
            rssi_scale: 0,
            rssi_invert: 0,
            rssi_offset: 0,
            fpv_cam_angle_degrees: 0,
            air_mode_activate_threshold: 0,
            spektrum_sat_bind: 0,
            mid_rc: 0,
            min_check: 0,
            max_check: 0,
            rx_min_us: 0,
            rx_max_us: 0,
        }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + Eq + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<RxConfig>();
        #[cfg(feature = "serde")]
        is_serde::<RxConfig>();
        #[cfg(feature = "storage")]
        is_storage::<RxConfig>();

        is_full_eq::<RadioType>();
        #[cfg(feature = "serde")]
        is_serde::<RadioType>();
        #[cfg(feature = "storage")]
        is_storage::<RadioType>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let config = RxConfig::new();
        assert_eq!(RadioType::Crsf, config.serial_rx_provider);
    }
}
