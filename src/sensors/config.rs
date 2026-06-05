#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SensorConfig {
    pub flags: u32,
}

#[allow(unused)]
impl SensorConfig {
    pub const GYRO: u32 = 1 << 0;
    pub const ACC: u32 = 1 << 1;
    pub const BAROMETER: u32 = 1 << 2;
    pub const MAGNETOMETER: u32 = 1 << 3;
    pub const SONAR: u32 = 1 << 4;
    pub const RANGEFINDER: u32 = 1 << 5;
    pub const GPS: u32 = 1 << 6;
    pub const GPS_MAGNETOMETER: u32 = 1 << 6;
    pub const OPTICAL_FLOW: u32 = 1 << 6;

    pub const fn new() -> Self {
        Self { flags: SensorConfig::GYRO | SensorConfig::ACC }
    }

    pub fn set(&mut self, flag: u32) {
        self.flags |= flag;
    }

    pub fn is_set(self, flag: u32) -> bool {
        self.flags & flag != 0
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn flags(self) -> u16 {
        self.flags as u16
    }
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for SensorConfig {}

impl Default for SensorConfig {
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
        is_full::<SensorConfig>();
        #[cfg(feature = "serde")]
        is_config::<SensorConfig>();
    }
    #[test]
    fn test_new() {
        let features = SensorConfig::default();
        assert!(features.is_set(SensorConfig::ACC));
        assert!(features.is_set(SensorConfig::GYRO));
    }
}
