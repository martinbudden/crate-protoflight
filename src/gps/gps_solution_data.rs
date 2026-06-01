#![allow(unused)]

use crate::gps::GpsDataPosition;

/// A value below 100 means great accuracy is possible with the GPS satellite constellation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GpsDilution {
    // positional DOP - 3D (* 100)
    pub positional: u16,
    // horizontal DOP - 2D (* 100)
    pub horizontal: u16,
    // vertical DOP   - 1D (* 100)
    pub vertical: u16,
}

impl GpsDilution {
    pub const fn new() -> Self {
        Self { positional: 0, horizontal: 0, vertical: 0 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GpsAccuracy {
    // horizontal accuracy in mm
    pub h_accuracy_mm: u32,
    // vertical accuracy in mm
    pub v_accuracy_mm: u32,
    // speed accuracy in mm/s
    pub s_accuracy_mm: u32,
    // heading accuracy in degrees * 1e-5
    pub heading_accuracy_degrees_x1e5: u32,
}

impl GpsAccuracy {
    pub const fn new() -> Self {
        Self { h_accuracy_mm: 0, v_accuracy_mm: 0, s_accuracy_mm: 0, heading_accuracy_degrees_x1e5: 0 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GpsVelocityNedCmps {
    /// North speed, in cm/s.
    pub north: i16,
    /// East speed, in cm/s.
    pub east: i16,
    /// Down speed, in cm/s.
    pub down: i16,
}

impl GpsVelocityNedCmps {
    pub const fn new() -> Self {
        Self { north: 0, east: 0, down: 0 }
    }
}
/// GPS date/time from NAV-PVT message.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GpsDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub millis: u16,
    pub valid: bool, // true when date/time rom GPS are valid
}

impl GpsDateTime {
    pub const fn new() -> Self {
        Self { year: 0, month: 0, day: 0, hour: 0, min: 0, sec: 0, millis: 0, valid: false }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpsSolutionData {
    pub llh: GpsDataPosition,
    pub dop: GpsDilution,
    pub accuracy: GpsAccuracy,
    pub velocity_ned: GpsVelocityNedCmps,
    // speed in cm/s
    pub speed3d_cmps: u16,
    // speed in cm/s
    pub ground_speed_cmps: u16,
    // degrees * 10
    pub ground_course_degrees_x10: u16,
    pub satellite_count: u8,
    pub time: u32,
    // interval between navigation solutions in ms
    pub navigation_interval_ms: u32,
    // GPS date/time from NAV-PVT
    pub date_time: GpsDateTime,
}

impl GpsSolutionData {
    pub const fn new() -> Self {
        Self {
            llh: GpsDataPosition::new(),
            dop: GpsDilution::new(),
            accuracy: GpsAccuracy::new(),
            velocity_ned: GpsVelocityNedCmps::new(),
            speed3d_cmps: 0,
            ground_speed_cmps: 0,
            ground_course_degrees_x10: 0,
            satellite_count: 0,
            time: 0,
            navigation_interval_ms: 0,
            date_time: GpsDateTime::new(),
        }
    }
}

impl Default for GpsSolutionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GpsSolutionDataAbridged {
    pub llh: GpsDataPosition,
    pub satellite_count: u8,
    // speed in cm/s
    pub ground_speed_cmps: u16,
    // degrees * 10
    pub ground_course_degrees_x10: u16,
    pub dop_positional: u16,
}

impl GpsSolutionDataAbridged {
    pub const fn new() -> Self {
        Self {
            llh: GpsDataPosition::new(),
            ground_speed_cmps: 0,
            ground_course_degrees_x10: 0,
            satellite_count: 0,
            dop_positional: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<GpsSolutionData>();
        is_full::<GpsSolutionDataAbridged>();
    }
}
