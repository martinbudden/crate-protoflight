use crate::magnetometer_sensors::MagnetometerType;

#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

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
    /// Magnetic alignment.
    pub alignment: u8,
    pub hardware: MagnetometerType,
    pub i2c_address: u8,
    pub zero: FlightDynamicsTrims,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for MagnetometerConfig {}

impl MagnetometerConfig {
    pub const fn new() -> Self {
        Self {
            alignment: 0, // mag alignment
            hardware: MagnetometerType::None,
            i2c_address: 0,
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
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<MagnetometerConfig>();
        #[cfg(feature = "serde")]
        is_config::<MagnetometerConfig>();
    }
}
