#![allow(unused)]

#[cfg(feature = "magnetometer")]
use crate::magnetometer_sensors::magnetometer_hmc5883::MagnetometerHmc5883;
use crate::{i2c_bus::SharedI2cBus, magnetometer_sensors::magnetometer_mock::MagnetometerMock};

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

// Type of magnetometer used/detected
#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MagnetometerType {
    #[default]
    Default = 0,
    None = 1,
    Hmc5883 = 2,
    Ak8975 = 3,
    Ak8963 = 4,
    Qmc5883 = 5,
    Lis2Mdl = 6,
    Lis3Mdl = 7,
    Mpu925xAk8963 = 8,
    Ist8310 = 9,
    Mmc560x = 10,
    Mock = 11,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for MagnetometerType {}

#[allow(unused)]
impl MagnetometerType {
    pub const COUNT: u8 = 11;

    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::None,
            2 => Self::Hmc5883,
            3 => Self::Ak8975,
            4 => Self::Ak8963,
            5 => Self::Qmc5883,
            6 => Self::Lis2Mdl,
            7 => Self::Lis3Mdl,
            8 => Self::Mpu925xAk8963,
            9 => Self::Ist8310,
            10 => Self::Mmc560x,
            11 => Self::Mock,
            _ => Self::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
pub struct MagnetometerMessage {
    pub quality: u16,
}

impl MagnetometerMessage {
    pub const fn new() -> Self {
        Self { quality: 0 }
    }
}

impl Default for MagnetometerMessage {
    fn default() -> Self {
        Self::new()
    }
}

/// The common interface for magnetometer.
pub trait RxMagnetometer {
    fn message(&self) -> MagnetometerMessage;
}

pub enum Magnetometer {
    Mock(MagnetometerMock),
    #[cfg(feature = "magnetometer")]
    Hmc5883(MagnetometerHmc5883),
}

impl Magnetometer {
    #[must_use]
    pub const fn new(magnetometer_type: MagnetometerType, i2c_bus: &'static SharedI2cBus) -> Option<Magnetometer> {
        match magnetometer_type {
            #[cfg(feature = "magnetometer")]
            MagnetometerType::Hmc5883 => Some(Self::Hmc5883(MagnetometerHmc5883::new(i2c_bus))),
            MagnetometerType::Mock => Some(Self::Mock(MagnetometerMock::new())),
            _ => None,
        }
    }
}

impl RxMagnetometer for Magnetometer {
    fn message(&self) -> MagnetometerMessage {
        match self {
            Self::Mock(magnetometer) => magnetometer.message(),
            #[cfg(feature = "magnetometer")]
            Self::Hmc5883(magnetometer) => magnetometer.message(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn _is_full_no_partial_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<MagnetometerMessage>();
        is_full::<MagnetometerType>();
        #[cfg(feature = "serde")]
        is_config::<MagnetometerType>();
    }
}
