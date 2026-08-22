#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CurrentSensorAdcConfig {
    /// scale the current sensor output voltage to milliamps. Value in mV/10A.
    pub scale: i16,
    // offset of the current sensor in mA
    pub offset_ma: i16,
}

#[cfg(feature = "serde")]
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

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CurrentSensorVirtualConfig {
    /// scale the throttle to centiamps, using a thrust linearization function.
    pub scale: i16,
    /// offset of the current sensor in centiamps (1/100th A).
    pub offset_centi_amps: i16,
}

#[cfg(feature = "serde")]
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
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CurrentMeterSource {
    #[default]
    None,
    Adc,
    Virtual,
    Esc,
    Msp,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for CurrentMeterSource {}

#[allow(unused)]
impl CurrentMeterSource {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Adc,
            2 => Self::Virtual,
            3 => Self::Esc,
            4 => Self::Msp,
            _ => Self::default(),
        }
    }

    #[must_use]
    pub fn try_from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Adc),
            2 => Some(Self::Virtual),
            3 => Some(Self::Esc),
            4 => Some(Self::Msp),
            _ => None,
        }
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CurrentSensorType {
    #[default]
    Virtual,
    Adc,
    Esc,
    Msp,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for CurrentSensorType {}

#[allow(unused)]
impl CurrentSensorType {
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

    #[must_use]
    pub fn try_from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Virtual),
            1 => Some(Self::Adc),
            2 => Some(Self::Esc),
            4 => Some(Self::Msp),
            _ => None,
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
        is_full::<CurrentSensorAdcConfig>();
        is_full::<CurrentSensorVirtualConfig>();
        is_full::<CurrentMeterSource>();
        is_full::<CurrentSensorType>();

        #[cfg(feature = "serde")]
        is_config::<CurrentSensorAdcConfig>();
        #[cfg(feature = "serde")]
        is_config::<CurrentSensorVirtualConfig>();
        #[cfg(feature = "serde")]
        is_config::<CurrentMeterSource>();
        #[cfg(feature = "serde")]
        is_config::<CurrentSensorType>();
    }
}
