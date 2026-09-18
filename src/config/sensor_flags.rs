#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct SensorFlags {
    pub flags: u32,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for SensorFlags {}

impl Default for SensorFlags {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl SensorFlags {
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
        Self { flags: SensorFlags::GYRO | SensorFlags::ACC }
    }

    pub fn set(&mut self, flag: u32) {
        self.flags |= flag;
    }

    pub fn is_set(self, flag: u32) -> bool {
        self.flags & flag != 0
    }

    pub fn flags(self) -> u32 {
        self.flags
    }

    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<SensorFlags>();
        #[cfg(feature = "serde")]
        is_serde::<SensorFlags>();
        #[cfg(feature = "storage")]
        is_storage::<SensorFlags>();
    }
    #[test]
    fn test_new() {
        let features = SensorFlags::default();
        assert!(features.is_set(SensorFlags::ACC));
        assert!(features.is_set(SensorFlags::GYRO));
    }
}
