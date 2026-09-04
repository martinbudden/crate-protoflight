#![cfg(feature = "autopilot")]

use sensor_fusion::AltitudeKalmanFilterExtendedf32;
#[cfg(any(feature = "gps", feature = "optical_flow"))]
use sensor_fusion::PositionKalmanFilterExtendedf32;

use super::altitude_dual_ring_pid::AltitudeDualRingPid;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Autopilot {
    pub altitude_controller: AltitudeDualRingPid,
    pub altitude_kalman_filter: AltitudeKalmanFilterExtendedf32,
    #[cfg(any(feature = "gps", feature = "optical_flow"))]
    pub position_kalman_filter: PositionKalmanFilterExtendedf32,
}

impl Default for Autopilot {
    fn default() -> Self {
        Self::new()
    }
}

impl Autopilot {
    pub fn new() -> Self {
        Self {
            altitude_controller: AltitudeDualRingPid::new(0.0),
            altitude_kalman_filter: AltitudeKalmanFilterExtendedf32::new(),
            #[cfg(any(feature = "gps", feature = "optical_flow"))]
            position_kalman_filter: PositionKalmanFilterExtendedf32::new(),
        }
    }
}
