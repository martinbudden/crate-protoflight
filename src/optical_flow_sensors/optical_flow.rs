#![allow(unused)]

use super::optical_flow_mock::OpticalFlowMock;
#[cfg(feature = "optical_flow")]
use super::optical_flow_mt::OpticalFlowMt;

#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

// Type of optical_flow used/detected
#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub enum OpticalFlowType {
    #[default]
    Default = 0,
    NoOpticalFlow = 1,
    Mt = 2,
    Upt1 = 3,
    Mock = 4,
}

#[allow(unused)]
impl OpticalFlowType {
    pub const COUNT: u8 = 4;

    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::NoOpticalFlow,
            2 => Self::Mt,
            3 => Self::Upt1,
            4 => Self::Mock,
            _ => Self::default(),
        }
    }
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for OpticalFlowType {}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
pub struct OpticalFlowMessage {
    pub quality: u16,
}

impl OpticalFlowMessage {
    pub const fn new() -> Self {
        Self { quality: 0 }
    }
}

impl Default for OpticalFlowMessage {
    fn default() -> Self {
        Self::new()
    }
}

/// The common interface for optical flow.
pub trait OpticalFlowDevice {
    fn message(&self) -> OpticalFlowMessage;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpticalFlow {
    Mock(OpticalFlowMock),
    #[cfg(feature = "optical_flow")]
    Mt(OpticalFlowMt),
}

impl OpticalFlow {
    #[must_use]
    pub const fn new(optical_flow_type: OpticalFlowType) -> Option<OpticalFlow> {
        match optical_flow_type {
            #[cfg(feature = "optical_flow")]
            OpticalFlowType::Mt => Some(Self::Mt(OpticalFlowMt::new())),
            OpticalFlowType::Mock => Some(Self::Mock(OpticalFlowMock::new())),
            _ => None,
        }
    }
}
impl OpticalFlowDevice for OpticalFlow {
    fn message(&self) -> OpticalFlowMessage {
        match self {
            Self::Mock(optical_flow) => optical_flow.message(),
            #[cfg(feature = "optical_flow")]
            Self::Mt(optical_flow) => optical_flow.message(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + MaxSize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}
    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + Eq + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<OpticalFlowMessage>();
        is_full_eq::<OpticalFlowType>();
    }
}
