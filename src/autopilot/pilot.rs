#![allow(unused)]

use sensor_fusion::AltitudeKalmanFilterf32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutoPilot {
    altitude_kalman_filter: AltitudeKalmanFilterf32,
}

impl AutoPilot {}
impl Default for AutoPilot {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoPilot {
    pub fn new() -> Self {
        Self { altitude_kalman_filter: AltitudeKalmanFilterf32::new() }
    }
}
