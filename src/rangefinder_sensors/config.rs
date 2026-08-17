use crate::rangefinder_sensors::RangefinderType;

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RangefinderConfig {
    pub hardware: RangefinderType,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for RangefinderConfig {}

#[allow(unused)]
impl RangefinderConfig {
    pub const fn new() -> Self {
        Self { hardware: RangefinderType::None }
    }
}

impl Default for RangefinderConfig {
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
        is_full::<RangefinderConfig>();
        #[cfg(feature = "serde")]
        is_config::<RangefinderConfig>();
    }
}
