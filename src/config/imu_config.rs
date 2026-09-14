#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};
#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct ImuConfig {
    // DCM filter proportional gain ( x 10000)
    pub imu_dcm_kp_x1e4: u16,
    // DCM filter integral gain ( x 10000)
    pub imu_dcm_ki_x1e4: u16,
    pub small_angle: u8,
    pub imu_process_denom: u8,
    // Magnetic declination in degrees * 10
    pub mag_declination_degrees_x10: i16,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for ImuConfig {}

impl Default for ImuConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ImuConfig {
    pub const fn new() -> Self {
        Self {
            imu_dcm_kp_x1e4: 2500,
            imu_dcm_ki_x1e4: 0,
            small_angle: 25,
            imu_process_denom: 2,
            mag_declination_degrees_x10: 0,
        }
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
        is_full::<ImuConfig>();
        #[cfg(feature = "serde")]
        is_serde::<ImuConfig>();
        #[cfg(feature = "storage")]
        is_storage::<ImuConfig>();
    }
    #[test]
    fn test_new() {
        let config = ImuConfig::new();
        assert_eq!(2500, config.imu_dcm_kp_x1e4);
    }
}
