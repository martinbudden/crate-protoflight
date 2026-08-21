use vqm::Vector3f32;

use crate::gps::{GpsData, GpsStatusData};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
#[cfg_attr(feature = "std", display("Gps{{yaw_rate: {yaw_heading_radians}}}"))]
pub struct GpsYawHeadingMessage {
    pub yaw_heading_radians: f32,
    pub delta_t: f32,
}

impl Default for GpsYawHeadingMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl GpsYawHeadingMessage {
    pub const fn new() -> Self {
        Self { yaw_heading_radians: 0.0, delta_t: 0.1 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum GpsMessage {
    Data(GpsData),
    Position(Vector3f32),
    Status(GpsStatusData),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_full_no_default<T: Sized + Send + Sync + Unpin + Copy + Clone + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<GpsYawHeadingMessage>();
        is_full_no_default::<GpsMessage>();
    }
}
