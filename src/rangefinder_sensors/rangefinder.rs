#![allow(unused)]

#[cfg(feature = "rangefinder")]
use super::rangefinder_hcsr04::RangefinderHcsr04;
use super::rangefinder_mock::RangefinderMock;

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

// Type of rangefinder used/detected
#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub enum RangefinderType {
    #[default]
    NoRangefinder = 0,
    Hcsr04 = 1,
    TfMini = 2,
    Tf02 = 3,
    Mtf01 = 4,
    Mtf02 = 5,
    Mtf01P = 6,
    Mtf02P = 7,
    TfNova = 8,
    NoopLoopF2 = 9,
    NoopLoopF2p = 10,
    NoopLoopF2ph = 11,
    NoopLoopF = 12,
    NoopLoopFp = 13,
    NoopLoopF2mini = 14,
    Mock = 255,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for RangefinderType {}

#[allow(unused)]
impl RangefinderType {
    pub const COUNT: u8 = 14;

    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NoRangefinder,
            1 => Self::Hcsr04,
            2 => Self::TfMini,
            3 => Self::Tf02,
            4 => Self::Mtf01,
            5 => Self::Mtf02,
            6 => Self::Mtf01P,
            7 => Self::Mtf02P,
            8 => Self::TfNova,
            9 => Self::NoopLoopF2,
            10 => Self::NoopLoopF2p,
            11 => Self::NoopLoopF2ph,
            12 => Self::NoopLoopF,
            13 => Self::NoopLoopFp,
            14 => Self::NoopLoopF2mini,
            _ => Self::default(),
        }
    }
}

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

/// The common interface for rangefinder.
pub trait RangefinderDevice {
    fn message(&self) -> RangefinderMessage;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rangefinder {
    Mock(RangefinderMock),
    #[cfg(feature = "rangefinder")]
    Hmc5883(RangefinderHcsr04),
}

impl Rangefinder {
    #[must_use]
    pub const fn new(rangefinder_type: RangefinderType) -> Option<Rangefinder> {
        match rangefinder_type {
            #[cfg(feature = "rangefinder")]
            RangefinderType::Hcsr04 => Some(Self::Hmc5883(RangefinderHcsr04::new())),
            RangefinderType::Mock => Some(Self::Mock(RangefinderMock::new())),
            _ => None,
        }
    }
}

impl RangefinderDevice for Rangefinder {
    fn message(&self) -> RangefinderMessage {
        match self {
            Self::Mock(rangefinder) => rangefinder.message(),
            #[cfg(feature = "rangefinder")]
            Self::Hmc5883(rangefinder) => rangefinder.message(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + Eq + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<RangefinderMessage>();
        is_full_eq::<RangefinderType>();
        #[cfg(feature = "serde")]
        is_full::<RangefinderType>();
    }
}
