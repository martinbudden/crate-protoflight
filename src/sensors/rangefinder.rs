#![cfg(feature = "rangefinder")]

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
#[cfg_attr(feature = "std", display("Range{{d:{distance_m}}}"))]
pub struct RangefinderMessage {
    pub distance_m: f32,
}

impl RangefinderMessage {
    pub const fn new() -> Self {
        Self { distance_m: 0.0 }
    }
}

impl Default for RangefinderMessage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RangefinderConfig {
    pub hardware: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for RangefinderConfig {}

#[allow(unused)]
impl RangefinderConfig {
    const NONE: u8 = 0;
    const HCSR04: u8 = 1;
    const TF_MINI: u8 = 2;
    const TF02: u8 = 3;
    const MTF01: u8 = 4;
    const MTF02: u8 = 5;
    const MTF01P: u8 = 6;
    const MTF02P: u8 = 7;
    const TF_NOVA: u8 = 8;
    const NOOP_LOOP_F2: u8 = 9;
    const NOOP_LOOP_F2P: u8 = 10;
    const NOOP_LOOP_F2PH: u8 = 11;
    const NOOP_LOOP_F: u8 = 12;
    const NOOP_LOOP_FP: u8 = 13;
    const NOOP_LOOP_F2MINI: u8 = 14;
    const UPT1: u8 = 15;

    pub const fn new() -> Self {
        Self { hardware: Self::NONE }
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
    fn _is_full_no_partial_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<RangefinderMessage>();
        is_full::<RangefinderConfig>();
        #[cfg(feature = "serde")]
        is_config::<RangefinderConfig>();
    }
}
