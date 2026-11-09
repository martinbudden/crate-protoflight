use vqm::Vector3f32;

use crate::gps::{Geodetic, GeographicCoordinate};

use super::{GpsSolution, GpsSolutionAbridged, GpsStatus};

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
    Solution(GpsSolution),
    //Position(Vector3f32),
    /// `timestamp_us`, `position_m`, `velocity_mps`.
    PositionVelocity(u64, Vector3f32, Vector3f32),
    Status(GpsStatus),
}

impl GpsMessage {
    /// Converts `GpsSolution` velocity in `i16` cmps to `f32` mps.
    pub fn from_gps_solution(gps_solution: &GpsSolution, initial_time_of_week_ms: u32,home: &Geodetic) -> GpsMessage {
        const CMPS_TO_MPS: f32 = 0.01;

        let time_us = u64::from(gps_solution.time_of_week_ms.wrapping_sub(initial_time_of_week_ms) * 1000);

        let geographic_coordinate = GeographicCoordinate::from_long_lat_alt(
            gps_solution.longitude_degrees_x1e7,
            gps_solution.latitude_degrees_x1e7,
            gps_solution.altitude_cm,
        );
        let position_m = home.position_relative_to_home_meters(geographic_coordinate);

        let velocity_mps = Vector3f32 {
            x: f32::from(gps_solution.velocity_north_cmps) * CMPS_TO_MPS,
            y: f32::from(gps_solution.velocity_east_cmps) * CMPS_TO_MPS,
            z: f32::from(gps_solution.velocity_down_cmps) * CMPS_TO_MPS,
        };
        GpsMessage::PositionVelocity(time_us, position_m, velocity_mps)
    }
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
