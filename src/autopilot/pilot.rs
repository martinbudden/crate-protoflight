#![cfg(feature = "autopilot")]

#[cfg(any(feature = "gps", feature = "optical_flow"))]
use sensor_fusion::KalmanFilterXYZWithSensorsf32;
use sensor_fusion::KalmanFilterZWithSensorsf32;

use super::dual_ring_pid_vertical::MultirotorAltitudeDualRingPid;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Autopilot {
    pub altitude_controller: MultirotorAltitudeDualRingPid,
    pub altitude_kalman_filter: KalmanFilterZWithSensorsf32,
    #[cfg(any(feature = "gps", feature = "optical_flow"))]
    pub position_kalman_filter: KalmanFilterXYZWithSensorsf32,
}

impl Default for Autopilot {
    fn default() -> Self {
        Self::new()
    }
}

impl Autopilot {
    pub fn new() -> Self {
        Self {
            altitude_controller: MultirotorAltitudeDualRingPid::new(0.0),
            altitude_kalman_filter: KalmanFilterZWithSensorsf32::new(),
            #[cfg(any(feature = "gps", feature = "optical_flow"))]
            position_kalman_filter: KalmanFilterXYZWithSensorsf32::new(),
        }
    }
}
