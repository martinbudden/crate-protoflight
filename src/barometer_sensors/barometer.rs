use crate::barometer_sensors::{BarometerDevice, BarometerMessage, barometer_mock::BarometerMock};
#[cfg(feature = "barometer")]
use crate::barometer_sensors::{barometer_bmp085::BarometerBmp085, barometer_dps310::BarometerDps310};

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
    pub const COUNT: u8 = 13;

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
pub enum Barometer {
    Mock(BarometerMock),
    #[cfg(feature = "barometer")]
    Bmp085(BarometerBmp085),
    #[cfg(feature = "barometer")]
    Dps310(BarometerDps310),
}

impl Barometer {
    #[must_use]
    pub const fn new(barometer_type: BarometerType) -> Option<Barometer> {
        match barometer_type {
            BarometerType::Mock => Some(Self::Mock(BarometerMock::new())),
            //#[cfg(feature = "barometer")]
            //BarometerType::Default => Some(Self::Mock(BarometerMock::new())),
            #[cfg(feature = "barometer")]
            BarometerType::Bmp085 => Some(Self::Bmp085(BarometerBmp085::new())),
            #[cfg(feature = "barometer")]
            BarometerType::Dsp310 => Some(Self::Dps310(BarometerDps310::new())),
            _ => None,
        }
    }
}
impl BarometerDevice for Barometer {
    async fn init(&mut self) -> Result<u32, ()> {
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
