#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};
#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct CurrentSensorAdcConfig {
    /// scale the current sensor output voltage to milliamps. Value in mV/10A.
    pub scale: i16,
    // offset of the current sensor in mA
    pub offset_ma: i16,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for CurrentSensorAdcConfig {}

impl Default for CurrentSensorAdcConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl CurrentSensorAdcConfig {
    pub const fn new() -> Self {
        Self {
            scale: 400, // 40mV/A
            offset_ma: 0,
        }
    }
}

impl CurrentSensorAdcConfig {
    #[allow(unused)]
    pub fn reading_to_centi_amps(self, reading: i16) -> i32 {
        const REFERENCE_VOLTAGE_MV: i32 = 3300;

        let millivolts = i32::from(reading) * REFERENCE_VOLTAGE_MV / 4096;
        // y =x /m + b. m is scale in (mV/10A) and b is offset in (mA)
        if self.scale == 0 { 0 } else { millivolts * 10000 / i32::from(self.scale) + i32::from(self.offset_ma) / 10 }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct CurrentSensorVirtualConfig {
    /// scale the throttle to centiamps, using a thrust linearization function.
    pub scale: i16,
    /// offset of the current sensor in centiamps (1/100th A).
    pub offset_centi_amps: i16,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for CurrentSensorVirtualConfig {}

impl Default for CurrentSensorVirtualConfig {
    fn default() -> Self {
        Self::new()
    }
}
impl CurrentSensorVirtualConfig {
    pub const fn new() -> Self {
        Self { scale: 0, offset_centi_amps: 0 }
    }
}

#[allow(unused)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum CurrentMeterSource {
    #[default]
    NoSource = 0,
    Adc = 1,
    Virtual = 2,
    Esc = 3,
    Msp = 4,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for CurrentMeterSource {}

impl_try_from_u8!(CurrentMeterSource);

impl CurrentMeterSource {
    /// Forgiving conversion, converts invalid values to default.
    #[allow(unused)]
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NoSource,
            1 => Self::Adc,
            2 => Self::Virtual,
            3 => Self::Esc,
            4 => Self::Msp,
            _ => Self::default(),
        }
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub enum CurrentSensorType {
    #[default]
    Virtual,
    Adc,
    Esc,
    Msp,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for CurrentSensorType {}

impl_try_from_u8!(CurrentSensorType);

#[allow(unused)]
impl CurrentSensorType {
    /// Forgiving conversion, converts invalid values to default.
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Virtual,
            1 => Self::Adc,
            2 => Self::Esc,
            3 => Self::Msp,
            _ => Self::default(),
        }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + Eq + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<CurrentSensorAdcConfig>();
        is_full::<CurrentSensorVirtualConfig>();
        is_full_eq::<CurrentMeterSource>();
        is_full_eq::<CurrentSensorType>();
    }
    #[cfg(feature = "serde")]
    #[test]
    fn serde_types() {
        is_serde::<CurrentSensorAdcConfig>();
        is_serde::<CurrentSensorVirtualConfig>();
        is_serde::<CurrentMeterSource>();
        is_serde::<CurrentSensorType>();
    }
    #[cfg(feature = "storage")]
    #[test]
    fn storage_types() {
        is_storage::<CurrentSensorAdcConfig>();
        is_storage::<CurrentSensorVirtualConfig>();
        is_storage::<CurrentMeterSource>();
        is_storage::<CurrentSensorType>();
    }
}
