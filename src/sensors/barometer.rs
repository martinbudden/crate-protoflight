#![allow(unused)]

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarometerData {
    pub pressure_pascals: f32,
    pub temperature_celsius: f32,
}
impl BarometerData {
    pub const fn new() -> Self {
        Self { pressure_pascals: 0.0, temperature_celsius: 0.0 }
    }
}

impl Default for BarometerData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<BarometerData>();
    }
}
