#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FeatureConfig {
    pub features: u32,
}

#[allow(unused)]
impl FeatureConfig {
    pub const RX_PPM: u32 = 1 << 0;
    pub const INFLIGHT_ACC_CAL: u32 = 1 << 2;
    pub const RX_SERIAL: u32 = 1 << 3;
    pub const MOTOR_STOP: u32 = 1 << 4;
    pub const SERVO_TILT: u32 = 1 << 5;
    pub const SOFTSERIAL: u32 = 1 << 6;
    pub const GPS: u32 = 1 << 7;
    pub const OPTICAL_FLOW: u32 = 1 << 8;
    pub const RANGEFINDER: u32 = 1 << 9;
    pub const TELEMETRY: u32 = 1 << 10;
    pub const THREE_D: u32 = 1 << 12;
    pub const RX_PARALLEL_PWM: u32 = 1 << 13;
    pub const RX_MSP: u32 = 1 << 14;
    pub const RSSI_ADC: u32 = 1 << 15;
    pub const LED_STRIP: u32 = 1 << 16;
    pub const DASHBOARD: u32 = 1 << 17;
    pub const OSD: u32 = 1 << 18;
    pub const CHANNEL_FORWARDING: u32 = 1 << 20;
    pub const TRANSPONDER: u32 = 1 << 21;
    pub const AIRMODE: u32 = 1 << 22;
    pub const RX_SPI: u32 = 1 << 25;
    //pub const SOFT_SPI:u32 =1 << 26; (removed)
    pub const ESC_SENSOR: u32 = 1 << 27;
    pub const ANTI_GRAVITY: u32 = 1 << 28;
    //pub const DYNAMIC_FILTER:u32 =1 << 29; (removed)

    pub const fn new() -> Self {
        Self { features: Self::RX_SERIAL | Self::AIRMODE | Self::ANTI_GRAVITY }
    }
    pub fn set(&mut self, feature: u32) {
        self.features |= feature;
    }

    pub fn is_set(self, feature: u32) -> bool {
        self.features & feature != 0
    }

    pub fn features(self) -> u32 {
        self.features
    }
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for FeatureConfig {}

impl Default for FeatureConfig {
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
        is_full::<FeatureConfig>();

        #[cfg(feature = "serde")]
        is_config::<FeatureConfig>();
    }
}
