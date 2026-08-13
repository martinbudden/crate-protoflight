#![cfg(feature = "barometer")]

use crate::barometer_sensors::{barometer_bmp085::BarometerBmp085, barometer_mock::BarometerMock};
#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum BarometerType {
    #[default]
    Default = 0,
    None = 1,
    Bmp085 = 2,
    Ms5611 = 3,
    Bmp280 = 4,
    Lps = 5,
    Qmp6988 = 6,
    Bmp388 = 7,
    Dsp310 = 8,
    Smpb02b = 9,
    Lps22Df = 10,
    Bmp580 = 11,
    Bmp581 = 12,
    Mock = 13,
}

#[allow(unused)]
impl BarometerType {
    pub const COUNT: u8 = 27;

    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::None,
            2 => Self::Bmp085,
            3 => Self::Ms5611,
            4 => Self::Bmp280,
            5 => Self::Lps,
            6 => Self::Qmp6988,
            7 => Self::Bmp388,
            8 => Self::Dsp310,
            9 => Self::Smpb02b,
            10 => Self::Lps22Df,
            11 => Self::Bmp580,
            12 => Self::Bmp581,
            13 => Self::Mock,
            _ => Self::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
#[cfg_attr(feature = "std", display("Baro{{a:{altitude_m}, p:{pressure_pascals}, t:{temperature_celsius}}}"))]
pub struct BarometerMessage {
    pub altitude_m: f32,
    pub altitude_m_i32: i32,
    pub pressure_pascals: f32,
    pub temperature_celsius: f32,
}

impl Default for BarometerMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl BarometerMessage {
    pub const fn new() -> Self {
        Self { altitude_m: 0.0, altitude_m_i32: 0, pressure_pascals: 0.0, temperature_celsius: 0.0 }
    }
}

/// The common interface for barometer.
pub trait RxBarometer {
    fn message(&self) -> BarometerMessage;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Barometer {
    Mock(BarometerMock),
    Bmp085(BarometerBmp085),
}

impl Barometer {
    #[must_use]
    pub const fn new(barometer_type: BarometerType) -> Option<Barometer> {
        match barometer_type {
            BarometerType::Bmp085 => Some(Self::Bmp085(BarometerBmp085::new())),
            BarometerType::Mock => Some(Self::Mock(BarometerMock::new())),
            _ => None,
        }
    }
}
impl RxBarometer for Barometer {
    fn message(&self) -> BarometerMessage {
        match self {
            Self::Mock(barometer) => barometer.message(),
            Self::Bmp085(barometer) => barometer.message(),
        }
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
    }
}
