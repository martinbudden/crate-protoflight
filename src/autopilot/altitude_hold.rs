#![allow(unused)]
use pidsk_controller::{PidControllerf32, PidGainsf32, PidLimitsf32};
use vqm::Quaternionf32;

/// Altitude hold uses a standard **Dual-Ring Cascaded PID Loop**.
///
/// Because a multirotor controls acceleration (via motor thrust) to change altitude,
/// a single PID loop often suffers from severe oscillation or sluggish response.
///
/// So, instead of mapping altitude error directly to throttle, there are two nested loops:
///
/// 1. Outer Loop (Altitude): Takes altitude error and outputs a target vertical speed (climb rate).
/// 2. Inner Loop (Vertical Speed): Takes vertical speed error and outputs the throttle offset.
///
/// Because vertical speed is the derivative of altitude, the outer loop does not need a Dterm,
/// since the inner loop Pterm is effectively the outer loop Dterm.
/// 
/// The inner loop incorporates some open-loop control via its Kterm (kick), so there is
/// an immediate response when its setpoint changes (no need to wait for the error to accumulate).
/// 
/// ```text
/// [Target Altitude]
///       │
///       ▼
///   ( + / - ) <─── [Current Altitude]
///       │
///       ▼
/// ┌───────────┐
/// │ PI-Loop   │ (Outer Loop)
/// └─────┬─────┘
///       │
/// [Target Vertical Speed]
///       │
///       ▼
///   ( + / - ) <─── [Current Vertical Speed]
///       │
///       ▼
/// ┌───────────┐
/// │ PIDK-Loop │ (Inner Loop)
/// └─────┬─────┘
///       │
///       ▼
/// [Throttle Offset] ──> + Base Throttle ──> Motors
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AltitudeController {
    /// Outer Loop: Inputs Target Altitude -> Output Target Vertical Speed.
    height_controller: PidControllerf32,
    /// Inner Loop: Input Target Vertical Speed -> Output Throttle Adjustment.
    speed_controller: PidControllerf32,

    // Core parameters
    /// max vertical speed (climb rate), m/s.
    max_vertical_speed_mps: f32,
    /// maximum allowed thrust adjustment.
    max_throttle_adjustment: f32,
    /// estimated throttle needed to hover.
    hover_throttle: f32,
}

impl Default for AltitudeController {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl AltitudeController {
    pub const fn new(hover_throttle: f32) -> Self {
        Self {
            // Initialize height controller (Outer Loop)
            // Only needs Proportional (kp) to map distance error to speed:
            // because the inner loop handles the physics of acceleration, 
            // the outer loop only needs Kp to calculate the vertical speed setpoint
            height_controller: PidControllerf32::new(1.0),
            // Initialize velocity controller (Inner Loop)
            // Highly reactive: utilizes kp, ki, and kd.
            // TODO: check default PID gains.
            speed_controller: PidControllerf32::with_gains(PidGainsf32 {
                kp: 2.5,
                ki: 0.05,
                kd: 0.05,
                ks: 0.0,
                kk: 0.0,
            }),
            max_vertical_speed_mps: 8.0, // 8.0 is required for simulator. Limit vertical speed to 2.5 m/s
            max_throttle_adjustment: 250.0,
            hover_throttle,
        }
    }
}

impl AltitudeController {
    pub fn set_altitude_setpoint(&mut self, altitude_setpoint: f32) {
        self.height_controller.set_setpoint(altitude_setpoint);
    }

    pub fn calculate_throttle_offset(
        &mut self,
        altitude: f32,
        vertical_speed: f32,
        orientation: Quaternionf32,
        delta_t: f32,
    ) -> f32 {
        let cos_tilt = orientation.cos_tilt();
        if cos_tilt < 0.0 {
            // craft is upside down, so cannot make adjustment
            return 0.0;
        }
        // --- STEP 1: Altitude Loop ---
        let vertical_speed_setpoint = self
            .height_controller
            .update_sp(altitude) // just call update_sp, since ki and kd are zero.
            .clamp(-self.max_vertical_speed_mps, self.max_vertical_speed_mps);

        // --- STEP 2: Vertical Speed Loop ---
        self.speed_controller.set_setpoint(vertical_speed_setpoint);

        // calculate throttle offset, adjusting for tilt angle.
        let cos_tilt_reciprocal = 1.0 / cos_tilt.clamp(0.1, 1.0);
        let throttle_offset =
            self.speed_controller.update(vertical_speed, delta_t) + self.hover_throttle * (cos_tilt_reciprocal - 1.0);

        throttle_offset.clamp(-self.max_throttle_adjustment, self.max_throttle_adjustment)
    }

    pub fn update(
        &mut self,
        altitude: f32,
        vertical_speed: f32,
        orientation: Quaternionf32,
        delta_t: f32,
    ) -> f32 {
        self.hover_throttle + self.calculate_throttle_offset(altitude, vertical_speed, orientation, delta_t)
    }
}


#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use vqm::Quaternion;

    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<AltitudeController>();
    }
    #[test]
    fn test_new() {
        let _altitude_hold = AltitudeController::new(0.0);
    }

    // A simple physics simulation of a multirotor's vertical motion
    struct MockMultirotor {
        altitude: f32,
        vertical_speed: f32,
        hover_throttle: f32,
        mass: f32,
    }

    impl MockMultirotor {
        const fn new(hover_throttle: f32) -> Self {
            Self {
                altitude: 0.0,
                vertical_speed: 0.0,
                hover_throttle,
                mass: 1.0, // Simplified mass constant
            }
        }

        // Simulates physics movement over a tiny slice of time
        fn step(&mut self, throttle: f32, dt: f32) {
            let motor_force = throttle - self.hover_throttle;
            let drag_force = 0.5 * self.vertical_speed;

            let acceleration = (motor_force - drag_force) / self.mass;

            // Update velocity_speed and altitude using Euler integration
            self.vertical_speed += acceleration * dt;
            self.altitude += self.vertical_speed * dt;
        }
    }

    #[test]
    fn test_altitude_hold_convergence() {
        let hover_throttle = 1500.0; // Steady state PWM mid-point
        let mut controller = AltitudeController::new(hover_throttle);
        let mut multirotor = MockMultirotor::new(hover_throttle);

        // --- BALANCED TUNING FOR UNIT SIMULATION ---
        //controller.height_controller.set_gains(PidGainsf32 { kp: 1.0, ki: 0.0, kd: 0.0, ks: 0.0, kk: 0.0 });
        // Strong P braking, minor I, strong D damping, no kick.
        controller.speed_controller.set_gains(PidGainsf32 { kp: 2.5, ki: 0.05, kd: 0.05, ks: 0.0, kk: 0.0 });

        let altitude_setpoint = 10.0; // Want to climb and hold at 10 meters
        controller.set_altitude_setpoint(altitude_setpoint);
        let delta_t = 0.0005; // 2kHz execution speed
        let orientation = Quaternionf32::default();

        // Simulate 5 seconds of real-time flight execution (10,000 loop cycles)
        let mut converged = false;
        for _ in 0..10_000 {
            let throttle = controller.update(
                multirotor.altitude,
                multirotor.vertical_speed,
                orientation,
                delta_t,
            );

            multirotor.step(throttle, delta_t);

            // Check if we reached the target closely and stabilized vertical velocity
            if (multirotor.altitude - altitude_setpoint).abs() < 0.05 && multirotor.vertical_speed.abs() < 0.01 {
                converged = true;
                break;
            }
        }

        assert!(
            converged,
            "Multirotor failed to settle at target altitude. Final Alt: {}, Vel: {}",
            multirotor.altitude, multirotor.vertical_speed
        );
    }
}
