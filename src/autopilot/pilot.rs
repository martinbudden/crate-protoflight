#![cfg(feature = "autopilot")]

#[cfg(any(feature = "gps", feature = "optical_flow"))]
use sensor_fusion::KalmanFilterXyzWithSensorsf32;
#[cfg(not(any(feature = "gps", feature = "optical_flow")))]
use sensor_fusion::KalmanFilterZWithSensorsf32;

#[allow(unused)]
use super::{
    dual_ring_pid_horizontal::HorizontalPositionDualRingPid, dual_ring_pid_vertical::MultirotorAltitudeDualRingPid,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Autopilot {
    pub altitude_controller: MultirotorAltitudeDualRingPid,

    // If we don't have either gps or optical flow, then we can't correct position in the xy plane,
    // so there is no point in having the overhead of a 3D Kalman filter, so we just use a `KalmanFilterZ`.
    #[cfg(not(any(feature = "gps", feature = "optical_flow")))]
    pub kalman_filter: KalmanFilterZWithSensorsf32,

    // If we have GPS or optical flow, then use a `KalmanFilterZ`.
    #[cfg(any(feature = "gps", feature = "optical_flow"))]
    pub kalman_filter: KalmanFilterXyzWithSensorsf32,
    #[cfg(any(feature = "gps", feature = "optical_flow"))]
    pub position_controller: HorizontalPositionDualRingPid,
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
            #[cfg(not(any(feature = "gps", feature = "optical_flow")))]
            kalman_filter: KalmanFilterZWithSensorsf32::new(),

            #[cfg(any(feature = "gps", feature = "optical_flow"))]
            kalman_filter: KalmanFilterXyzWithSensorsf32::new(),
            #[cfg(any(feature = "gps", feature = "optical_flow"))]
            position_controller: HorizontalPositionDualRingPid::new(),
        }
    }
}
