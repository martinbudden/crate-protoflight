#![cfg(feature = "barometer")]

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
#[cfg_attr(feature = "std", display("Baro{{a:{altitude_m}, p:{pressure_pascals}, t:{temperature_celsius}}}"))]
pub struct BarometerMessage {
    pub altitude_m: f32,
    pub pressure_pascals: f32,
    pub temperature_celsius: f32,
}

impl BarometerMessage {
    pub const fn new() -> Self {
        Self { altitude_m: 0.0, pressure_pascals: 0.0, temperature_celsius: 0.0 }
    }
}

impl Default for BarometerMessage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BarometerConfig {
    pub hardware: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for BarometerConfig {}

#[allow(unused)]
impl BarometerConfig {
    const DEFAULT: u8 = 0;
    const NONE: u8 = 1;
    const BMP085: u8 = 2;
    const MS5611: u8 = 3;
    const BMP280: u8 = 4;
    const LPS: u8 = 5;
    const QMP6988: u8 = 6;
    const BMP388: u8 = 7;
    const DPS310: u8 = 8;
    const SMPB_02B: u8 = 9;
    const LPS22DF: u8 = 10;
    const BMP580: u8 = 11;
    const BMP581: u8 = 12;
    const VIRTUAL: u8 = 13;

    pub const fn new() -> Self {
        Self { hardware: Self::DEFAULT }
    }
}

impl Default for BarometerConfig {
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
        is_full::<BarometerMessage>();
        is_full::<BarometerConfig>();
        #[cfg(feature = "serde")]
        is_config::<BarometerConfig>();
    }
}
