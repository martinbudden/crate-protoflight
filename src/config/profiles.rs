#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct SchemaVersion {
    pub version: u8,
}

#[cfg(feature = "storage")]
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
        is_full::<SchemaVersion>();
        #[cfg(feature = "serde")]
        is_serde::<SchemaVersion>();
        #[cfg(feature = "storage")]
        is_storage::<SchemaVersion>();
    }
}
