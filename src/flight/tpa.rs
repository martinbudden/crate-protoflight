#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

/// Configuration data for Throttle PID Attenuation (TPA),
/// Allows dynamic adjustment of the PID gains according to the throttle value.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct TpaConfig {
    pub mode: TpaMode,
    pub rate: u8,
    pub breakpoint: u16,
    pub low_rate: i8,
    pub low_always: u8,
    pub low_breakpoint: u16,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for TpaConfig {}

impl Default for TpaConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TpaConfig {
    pub const fn new() -> Self {
        Self { mode: TpaMode::D, rate: 65, breakpoint: 1350, low_rate: 20, low_always: 0, low_breakpoint: 1050 }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub enum TpaMode {
    P = 0,
    #[default]
    D = 1,
    Pds = 2,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for TpaMode {}

impl_try_from_u8!(TpaMode);

#[allow(unused)]
impl TpaMode {
    /// Forgiving conversion from u8 to `TpaMode`, converts invalid values to default.
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::P,
            1 => Self::D,
            2 => Self::Pds,
            _ => Self::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tpa {
    pub value: f32,
    pub multiplier: f32,
    pub breakpoint: f32,
}

impl Default for Tpa {
    fn default() -> Self {
        Self::new()
    }
}

impl Tpa {
    pub const fn new() -> Self {
        Self { value: 1.0, multiplier: 0.0, breakpoint: 0.0 }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + Eq + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full_eq::<TpaMode>();
        is_full::<TpaConfig>();
        is_full::<Tpa>();
    }
    #[cfg(feature = "serde")]
    #[test]
    fn serde_types() {
        is_serde::<TpaConfig>();
    }
    #[cfg(feature = "storage")]
    #[test]
    fn storage_types() {
        is_storage::<TpaConfig>();
    }
}
