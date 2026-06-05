#![cfg(feature = "barometer")]

#[derive(Clone, Copy, Debug, derive_more::Display, PartialEq)]
#[display("Baro{{a:{altitude_m}, p:{pressure_pascals}, t:{temperature_celsius}}}")]
pub struct BarometerData {
    pub altitude_m: f32,
    pub pressure_pascals: f32,
    pub temperature_celsius: f32,
}

impl BarometerData {
    pub const fn new() -> Self {
        Self { altitude_m: 0.0, pressure_pascals: 0.0, temperature_celsius: 0.0 }
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
