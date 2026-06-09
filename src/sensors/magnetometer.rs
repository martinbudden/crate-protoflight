#![cfg(feature = "magnetometer")]
#![allow(unused)]

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
pub struct MagnetometerData {
    pub quality: u16,
}

impl MagnetometerData {
    pub const fn new() -> Self {
        Self { quality: 0 }
    }
}

impl Default for MagnetometerData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FlightDynamicsTrims {
    pub roll: u16,
    pub pitch: u16,
    pub yaw: u16,
    pub calibration_completed: u16,
}
#[cfg(feature = "serde")]
impl PostcardValue<'_> for FlightDynamicsTrims {}

#[allow(unused)]
impl FlightDynamicsTrims {
    pub const fn new() -> Self {
        Self { roll: 0, pitch: 0, yaw: 0, calibration_completed: 0 }
    }
}

impl Default for FlightDynamicsTrims {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MagnetometerConfig {
    pub alignment: u8, // mag alignment
    pub hardware: u8,  // Which mag hardware to use on boards with more than one device
    pub bus_type: u8,
    pub i2c_device: u8,
    pub i2c_address: u8,
    pub spi_device: u8,
    pub spi_csn: u8,
    pub interrupt_tag: u8,
    pub zero: FlightDynamicsTrims,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for MagnetometerConfig {}

#[allow(unused)]
impl MagnetometerConfig {
    const DEFAULT: u8 = 0;
    const NONE: u8 = 1;
    const HMC5883: u8 = 2;
    const AK8975: u8 = 3;
    const AK8963: u8 = 4;
    const QMC5883: u8 = 5;
    const LIS2MDL: u8 = 6;
    const LIS3MDL: u8 = 7;
    const MPU925X_AK8963: u8 = 8;
    const IST8310: u8 = 9;
    const MMC560X: u8 = 10;

    pub const fn new() -> Self {
        Self {
            alignment: 0, // mag alignment
            hardware: 0,  // Which mag hardware to use on boards with more than one device
            bus_type: 0,
            i2c_device: 0,
            i2c_address: 0,
            spi_device: 0,
            spi_csn: 0,
            interrupt_tag: 0,
            zero: FlightDynamicsTrims::new(),
        }
    }
}

impl Default for MagnetometerConfig {
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
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<MagnetometerData>();
        is_full::<MagnetometerConfig>();
        #[cfg(feature = "serde")]
        is_config::<MagnetometerConfig>();
    }
}
