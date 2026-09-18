use super::OpticalFlowType;

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct OpticalFlowConfig {
    pub hardware: OpticalFlowType,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for OpticalFlowConfig {}

impl Default for OpticalFlowConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl OpticalFlowConfig {
    pub const fn new() -> Self {
        Self { hardware: OpticalFlowType::NoOpticalFlow }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<OpticalFlowConfig>();
        #[cfg(feature = "serde")]
        is_serde::<OpticalFlowConfig>();
    }
}
