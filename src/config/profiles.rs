#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SchemaVersion {
    pub version: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for SchemaVersion {}

impl SchemaVersion {
    pub const fn new() -> Self {
        Self { version: 0 }
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PidProfile {
    pub profile: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for PidProfile {}

impl PidProfile {
    pub const _COUNT: usize = 4;

    pub const fn new() -> Self {
        Self { profile: 0 }
    }
}

impl Default for PidProfile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RatesProfile {
    pub profile: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for RatesProfile {}

impl RatesProfile {
    pub const _COUNT: usize = 4;

    pub const fn new() -> Self {
        Self { profile: 0 }
    }
}

impl Default for RatesProfile {
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
        is_full::<SchemaVersion>();
        #[cfg(feature = "serde")]
        is_config::<SchemaVersion>();
        is_full::<PidProfile>();
        #[cfg(feature = "serde")]
        is_config::<PidProfile>();
        is_full::<RatesProfile>();
        #[cfg(feature = "serde")]
        is_config::<RatesProfile>();
    }
}
