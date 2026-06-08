#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg(feature = "serde")]
impl PostcardValue<'_> for ImuConfig {}

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

impl Default for ImuConfig {
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
        is_full::<ImuConfig>();
        #[cfg(feature = "serde")]
        is_config::<ImuConfig>();
    }
    #[test]
    fn test_new() {
        let config = ImuConfig::new();
        assert_eq!(2500, config.imu_dcm_kp_x1e4);
    }
}
