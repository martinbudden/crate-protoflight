#![allow(unused)]
use sensor_fusion::AltitudeKalmanFilterf32;
#[cfg(feature = "gps")]
use sensor_fusion::PositionKalmanFilterf32;

use crate::autopilot::altitude_dual_ring_pid::AltitudeDualRingPid;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Autopilot {
    pub altitude_kalman_filter: AltitudeKalmanFilterf32,
    #[cfg(feature = "gps")]
    pub position_kalman_filter: PositionKalmanFilterf32,
    pub altitude_controller: AltitudeDualRingPid,
}

impl Default for Autopilot {
    fn default() -> Self {
        Self::new()
    }
}

impl Autopilot {
    pub const fn new() -> Self {
        Self {
            altitude_kalman_filter: AltitudeKalmanFilterf32::new(),
            #[cfg(feature = "gps")]
            position_kalman_filter: PositionKalmanFilterf32::new(),
            altitude_controller: AltitudeDualRingPid::new(0.0),
        }
    }
}
