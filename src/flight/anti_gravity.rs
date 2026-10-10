#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct AntiGravityConfig {
    pub cutoff_hz: u8,
    pub p_gain: u8,
    pub i_gain: u8,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for AntiGravityConfig {}

impl Default for AntiGravityConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl AntiGravityConfig {
    pub const fn new() -> Self {
        Self { cutoff_hz: 5, p_gain: 100, i_gain: 80 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AntiGravity {
    pub p_gain: f32,
    pub i_gain: f32,
}

impl Default for AntiGravity {
    fn default() -> Self {
        Self::new()
    }
}

impl AntiGravity {
    pub const KI: f32 = 0.0;

    pub const fn new() -> Self {
        Self { p_gain: 0.0, i_gain: 0.0 }
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
        is_full_eq::<AntiGravity>();
        is_full::<AntiGravityConfig>();
    }
    #[cfg(feature = "serde")]
    #[test]
    fn serde_types() {
        is_serde::<AntiGravityConfig>();
    }
    #[cfg(feature = "storage")]
    #[test]
    fn storage_types() {
        is_storage::<AntiGravityConfig>();
    }
}
