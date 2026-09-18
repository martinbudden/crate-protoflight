//#[cfg(feature = "barometer")]
#[allow(unused)]
use super::{
    BarometerDevice, BarometerMessage, barometer_bmp085::BarometerBmp085, barometer_dps310::BarometerDps310,
    barometer_mock::BarometerMock,
};
use crate::i2c_bus::{I2cError, SharedI2cBus};

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

#[allow(missing_docs)]
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub enum BarometerType {
    #[default]
    Default = 0,
    NoBarometer = 1,
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

#[cfg(feature = "storage")]
impl PostcardValue<'_> for BarometerType {}

#[allow(unused)]
impl BarometerType {
    /// Forgiving conversion, converts invalid values to default.
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::NoBarometer,
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

#[allow(unused)]
#[derive(Debug)]
pub enum BarometerError<E> {
    I2c(E),
    InvalidChipId { expected: u8, actual: u8 },
    InvalidCalibration,
    InvalidData,
}

//pub type Dps310Error = BarometerError<<I2cDeviceBlocking as embedded_hal::i2c::ErrorType>::Error>;
//pub type Bmp085Error = BarometerError<<I2cDeviceBlocking as embedded_hal::i2c::ErrorType>::Error>;

pub type BarometerI2cError = BarometerError<I2cError>;

pub enum Barometer {
    Mock(BarometerMock),
    #[cfg(feature = "barometer")]
    Bmp085(BarometerBmp085),
    #[cfg(feature = "barometer")]
    Dps310(BarometerDps310),
}

impl Barometer {
    #[allow(unused)]
    #[must_use]
    pub const fn new(barometer_type: BarometerType, i2c_bus: &'static SharedI2cBus) -> Option<Barometer> {
        match barometer_type {
            BarometerType::Mock => Some(Self::Mock(BarometerMock::new())),
            //#[cfg(feature = "barometer")]
            //BarometerType::Default => Some(Self::Mock(BarometerMock::new(i2c_bus))),
            #[cfg(feature = "barometer")]
            BarometerType::Bmp085 => Some(Self::Bmp085(BarometerBmp085::new(i2c_bus))),
            #[cfg(feature = "barometer")]
            BarometerType::Dsp310 => Some(Self::Dps310(BarometerDps310::new(i2c_bus))),
            _ => None,
        }
    }
}

impl BarometerDevice for Barometer {
    async fn init(&mut self) -> Result<u32, BarometerI2cError> {
        match self {
            Self::Mock(barometer) => barometer.init().await,
            #[cfg(feature = "barometer")]
            Self::Bmp085(barometer) => barometer.init().await,
            #[cfg(feature = "barometer")]
            Self::Dps310(barometer) => barometer.init().await,
        }
    }
    async fn make_reading(&mut self) {
        match self {
            Self::Mock(barometer) => barometer.make_reading().await,
            #[cfg(feature = "barometer")]
            Self::Bmp085(barometer) => barometer.make_reading().await,
            #[cfg(feature = "barometer")]
            Self::Dps310(barometer) => barometer.make_reading().await,
        }
    }

    fn message(&self) -> BarometerMessage {
        match self {
            Self::Mock(barometer) => barometer.message(),
            #[cfg(feature = "barometer")]
            Self::Bmp085(barometer) => barometer.message(),
            #[cfg(feature = "barometer")]
            Self::Dps310(barometer) => barometer.message(),
        }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + Eq + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full_eq::<BarometerType>();
        #[cfg(feature = "serde")]
        is_serde::<BarometerType>();
        #[cfg(feature = "storage")]
        is_storage::<BarometerType>();
    }
}
