use crate::barometer_sensors::SharedI2cBus;
use crate::magnetometer_sensors::{MagnetometerDevice, MagnetometerHmc5883, MagnetometerI2cError, MagnetometerMock};

use embassy_time::{Duration, Instant};

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

// Type of magnetometer used/detected
#[allow(missing_docs)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub enum MagnetometerType {
    #[default]
    Default = 0,
    NoMagnetometer = 1,
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

#[cfg(feature = "storage")]
impl PostcardValue<'_> for MagnetometerType {}

#[allow(unused)]
impl MagnetometerType {
    /// Forgiving conversion from u8 to `MagnetometerType`, converts invalid values to default.
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::NoMagnetometer,
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

pub enum Magnetometer {
    Mock(MagnetometerMock),
    Hmc5883(MagnetometerHmc5883),
}

impl Magnetometer {
    #[allow(unused)]
    #[must_use]
    pub const fn new(magnetometer_type: MagnetometerType, i2c_bus: &'static SharedI2cBus) -> Option<Magnetometer> {
        match magnetometer_type {
            MagnetometerType::Hmc5883 => Some(Self::Hmc5883(MagnetometerHmc5883::new(i2c_bus))),
            MagnetometerType::Mock => Some(Self::Mock(MagnetometerMock::new())),
            _ => None,
        }
    }
}

impl MagnetometerDevice for Magnetometer {
    async fn init(&mut self) -> Result<Duration, MagnetometerI2cError> {
        match self {
            Self::Mock(magnetometer) => magnetometer.init().await,
            Self::Hmc5883(magnetometer) => magnetometer.init().await,
        }
    }
    async fn make_reading(&mut self) -> Result<(Instant, [i16; 3]), MagnetometerI2cError> {
        match self {
            Self::Mock(magnetometer) => magnetometer.make_reading().await,
            Self::Hmc5883(magnetometer) => magnetometer.make_reading().await,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + Eq + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full_eq::<MagnetometerType>();
        #[cfg(feature = "serde")]
        is_serde::<MagnetometerType>();
        #[cfg(feature = "storage")]
        is_storage::<MagnetometerType>();
    }
}
