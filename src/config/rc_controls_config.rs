#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct RcControlsConfig {
    pub deadband: u8,
    pub yaw_deadband: u8,         // invert the serial RX protocol compared to its default setting.
    pub yaw_control_reversed: u8, // allow rx to operate in half duplex mode on STM32 F4, ignored for F1 and F3.
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for RcControlsConfig {}

impl Default for RcControlsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RcControlsConfig {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self { deadband: 0, yaw_deadband: 0, yaw_control_reversed: 0 }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<RcControlsConfig>();
        #[cfg(feature = "serde")]
        is_serde::<RcControlsConfig>();
        #[cfg(feature = "storage")]
        is_storage::<RcControlsConfig>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let config = RcControlsConfig::new();
        assert_eq!(0, config.deadband);
    }
}
