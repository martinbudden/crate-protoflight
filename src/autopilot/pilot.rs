#![cfg(feature = "autopilot")]

use radio_controllers::RcMode;
use vqm::UnitQuaternionf32;

use crate::{
    flight::{RcControls, RxMessage},
    tasks::FailsafeState,
};

#[cfg(not(any(feature = "gps", feature = "optical_flow")))]
pub type KalmanState = sensor_fusion::KalmanStateZ;
#[cfg(any(feature = "gps", feature = "optical_flow"))]
pub type KalmanState = sensor_fusion::KalmanStateXyz;

#[allow(unused)]
use super::{
    dual_ring_pid_horizontal::HorizontalPositionDualRingPid, dual_ring_pid_vertical::MultirotorAltitudeDualRingPid,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Autopilot {
    pub altitude_controller: MultirotorAltitudeDualRingPid,

    #[cfg(any(feature = "gps", feature = "optical_flow"))]
    pub position_controller: HorizontalPositionDualRingPid,
    altitude_hold_mode_on: bool,
    position_hold_mode_on: bool,
}

impl Default for Autopilot {
    fn default() -> Self {
        Self::new()
    }
}

impl Autopilot {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            altitude_controller: MultirotorAltitudeDualRingPid::new(0.0),
            #[cfg(any(feature = "gps", feature = "optical_flow"))]
            position_controller: HorizontalPositionDualRingPid::new(),
            altitude_hold_mode_on: false,
            position_hold_mode_on: false,
        }
    }
}

impl Autopilot {
    pub fn update(
        &mut self,
        rx_message: Option<RxMessage>,
        failsafe_state: FailsafeState,
        kalman_state: &KalmanState,
        orientation: UnitQuaternionf32,
        delta_t: f32,
    ) -> Option<RcControls> {
        // TODO: handle failsafe state
        _ = failsafe_state;
        // Check if the rc_modes have changed.
        if let Some(rx_message) = rx_message {
            let rc_modes = rx_message.rc_modes;
            self.altitude_hold_mode_on = rc_modes.test(RcMode::ALTITUDE_HOLD);
            #[cfg(feature = "gps")]
            {
                self.position_hold_mode_on = rc_modes.test(RcMode::POSITION_HOLD)
                    | rc_modes.test(RcMode::GPS_RESCUE)
                    | rc_modes.test(RcMode::AUTOPILOT);
            }
        }
        if self.altitude_hold_mode_on {
            let (altitude_m, vertical_speed_mps) = (-kalman_state.pos_z_m(), -kalman_state.vel_z_mps());

            let throttle_stick = self.altitude_controller.update(altitude_m, vertical_speed_mps, orientation, delta_t);

            return Some(RcControls { throttle_stick, ..Default::default() });
        }
        None
    }
}
