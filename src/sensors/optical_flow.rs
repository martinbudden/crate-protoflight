#![cfg(feature = "optical_flow")]
#![allow(unused)]

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
pub struct OpticalFlowData {
    pub quality: u16,
}

impl OpticalFlowData {
    pub const fn new() -> Self {
        Self { quality: 0 }
    }
}

impl Default for OpticalFlowData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OpticalFlowConfig {
    pub hardware: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for OpticalFlowConfig {}

#[allow(unused)]
impl OpticalFlowConfig {
    const NONE: u8 = 0;
    const MT: u8 = 1;
    const UPT1: u8 = 2;

    pub const fn new() -> Self {
        Self { hardware: Self::NONE }
    }
}

impl Default for OpticalFlowConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn _is_full_no_partial_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<OpticalFlowData>();
        is_full::<OpticalFlowConfig>();
        #[cfg(feature = "serde")]
        is_config::<OpticalFlowConfig>();
    }
}
