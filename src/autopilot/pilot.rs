#![allow(unused)]

use sensor_fusion::AltitudeKalmanFilterf32;

use crate::autopilot::altitude_hold::AltitudeController;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Autopilot {
    pub altitude_kalman_filter: AltitudeKalmanFilterf32,
    pub altitude_controller: AltitudeController,
}

impl Autopilot {}
impl Default for Autopilot {
    fn default() -> Self {
        Self::new()
    }
}

impl Autopilot {
    pub const fn new() -> Self {
        Self { 
            altitude_kalman_filter: AltitudeKalmanFilterf32::new(),
            altitude_controller: AltitudeController::new(0.0),
        }
    }
}
