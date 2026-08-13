use crate::barometer_sensors::barometer::BarometerType;

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BarometerConfig {
    pub hardware: BarometerType,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for BarometerConfig {}

impl Default for BarometerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl BarometerConfig {
    #[cfg(feature = "std")]
    pub const fn new() -> Self {
        Self { hardware: BarometerType::Mock }
    }
    #[cfg(not(feature = "std"))]
    pub const fn new() -> Self {
        Self { hardware: BarometerType::Default }
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
        is_full::<BarometerConfig>();
        #[cfg(feature = "serde")]
        is_config::<BarometerConfig>();
    }
}
