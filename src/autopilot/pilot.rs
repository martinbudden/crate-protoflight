#![cfg(feature = "autopilot")]

use sensor_fusion::AltitudeKalmanFilterf32;
#[cfg(any(feature = "gps", feature = "optical_flow"))]
use sensor_fusion::PositionKalmanFilterf32;

use crate::autopilot::altitude_dual_ring_pid::AltitudeDualRingPid;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Autopilot {
    pub altitude_controller: AltitudeDualRingPid,
    pub altitude_kalman_filter: AltitudeKalmanFilterf32,
    #[cfg(any(feature = "gps", feature = "optical_flow"))]
    pub position_kalman_filter: PositionKalmanFilterf32,
}

impl Default for Autopilot {
    fn default() -> Self {
        Self::new()
    }
}

impl Autopilot {
    pub const fn new() -> Self {
        Self {
            altitude_controller: AltitudeDualRingPid::new(0.0),
            altitude_kalman_filter: AltitudeKalmanFilterf32::new(),
            #[cfg(any(feature = "gps", feature = "optical_flow"))]
            position_kalman_filter: PositionKalmanFilterf32::new(),
        }
    }
}
