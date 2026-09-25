use super::barometer::BarometerType;

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct BarometerConfig {
    pub hardware: BarometerType,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for BarometerConfig {}

impl Default for BarometerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl BarometerConfig {
    pub const fn new() -> Self {
        Self { hardware: BarometerType::Default }
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
        is_full::<BarometerConfig>();
        #[cfg(feature = "serde")]
        is_serde::<BarometerConfig>();
        #[cfg(feature = "storage")]
        is_storage::<BarometerConfig>();
    }
}
