#![allow(unused)]

#[derive(Clone, Copy, Debug, derive_more::Display, PartialEq)]
#[display("Range{{d:{distance_m}}}")]
pub struct RangefinderData {
    pub distance_m: f32,
}

impl RangefinderData {
    pub const fn new() -> Self {
        Self { distance_m: 0.0 }
    }
}

impl Default for RangefinderData {
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

    #[test]
    fn normal_types() {
        is_full::<RangefinderData>();
    }
}
