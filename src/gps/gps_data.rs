#![allow(unused)]

use vqm::Vector3df32;

use crate::gps::{GeographicCoordinate, GpsSolutionData};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpsPosition {
    pub position: Vector3df32,
}

impl GpsPosition {
    pub const fn new() -> Self {
        Self { position: Vector3df32 { x: 0.0, y: 0.0, z: 0.0 } }
    }
}

impl Default for GpsPosition {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpsDataPosition {
    pub longitude_degrees_x1e7: i32,
    pub latitude_degrees_x1e7: i32,
    pub altitude_cm: i32,
}

impl GpsDataPosition {
    pub const fn new() -> Self {
        Self { longitude_degrees_x1e7: 0, latitude_degrees_x1e7: 0, altitude_cm: 0 }
    }
}

impl Default for GpsDataPosition {
    fn default() -> Self {
        Self::new()
    }
}

impl From<GpsDataPosition> for GeographicCoordinate {
    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn from(position: GpsDataPosition) -> Self {
        Self {
            longitude_degrees: (position.longitude_degrees_x1e7 as f32) * 1e-7,
            latitude_degrees: (position.latitude_degrees_x1e7 as f32) * 1e-7,
            altitude_meters: (position.altitude_cm as f32) * 0.1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpsData {
    pub position: GpsDataPosition,
    pub distance_to_home_meters: f32,
    pub bearing_to_home_degrees: f32,
    pub distance_flown_meters: f32,
    pub time_of_week_ms: u32,
    pub velocity_north_cmps: i16,
    pub velocity_east_cmps: i16,
    pub velocity_down_cmps: i16,
    pub speed3d_cmps: i16,
    pub ground_speed_cmps: i16,
    pub heading_deci_degrees: i16,
    pub dilution_of_precision_positional: i16,
    pub satellite_count: u8,
    pub fix: u8,
    pub is_healthy: u8,
    pub update: u8,
}

impl GpsData {
    const FIX_HOME: u8 = 0x01;
    const FIX: u8 = 0x02;
    const FIX_EVER: u8 = 0x04;

    pub const fn new() -> Self {
        Self {
            position: GpsDataPosition::new(),
            distance_to_home_meters: 0.0,
            bearing_to_home_degrees: 0.0,
            distance_flown_meters: 0.0,
            time_of_week_ms: 0,
            velocity_north_cmps: 0,
            velocity_east_cmps: 0,
            velocity_down_cmps: 0,
            speed3d_cmps: 0,
            ground_speed_cmps: 0,
            heading_deci_degrees: 0,
            dilution_of_precision_positional: 0,
            satellite_count: 0,
            fix: 0,
            is_healthy: 0,
            update: 0,
        }
    }
}

impl Default for GpsData {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Clone, Copy, Debug, derive_more::Display, PartialEq)]
#[display("Gps{{yaw_rate:{yaw_heading_radians}}}")]
pub struct GpsYawHeadingData {
    pub yaw_heading_radians: f32,
    pub delta_t: f32,
}

impl GpsYawHeadingData {
    pub const fn new() -> Self {
        Self { yaw_heading_radians: 0.0, delta_t: 0.1 }
    }
}

impl Default for GpsYawHeadingData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum GpsDataItem {
    Gps(GpsData),
    GpsPosition(GpsPosition),
    GpsSolution(GpsSolutionData),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<GpsData>();
        is_full::<GpsData>();
        is_full::<GpsDataPosition>();
        is_full::<GpsYawHeadingData>();
    }
}
