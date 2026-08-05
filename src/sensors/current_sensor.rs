#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CurrentMeterSource {
    #[default]
    None,
    Adc,
    Virtual,
    Esc,
    Msp,
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CurrentSensorType {
    #[default]
    Virtual,
    Adc,
    Esc,
    Msp,
}

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

impl CurrentSensorVirtualConfig {
    pub const fn new() -> Self {
        Self { scale: 0, offset_centi_amps: 0 }
    }
}

impl Default for CurrentSensorVirtualConfig {
    fn default() -> Self {
        Self::new()
    }
}
