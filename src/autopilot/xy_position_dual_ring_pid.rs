#![allow(unused)]
use pidsk_controller::{PidControllerf32, PidGainsf32};

/// ```text
/// [Target Position (X, Y)]
///       │
///       ▼
///   ( + / - ) <─── [Current Position]
///       │
///       ▼
/// ┌───────────┐
/// │ P-Loop    │ (Outer Loop)
/// └─────┬─────┘
///       │
/// [Target Ground Velocity (Vx, Vy)]
///       │
///       ▼
///   ( + / - ) <─── [Current Ground Velocity]
///       │
///       ▼
/// ┌───────────┐
/// │ PID-Loop  │ (Inner Loop)
/// └─────┬─────┘
///       │
///       ▼
/// [        ]
///
/// Since velocity is the first derivative of position the outer loop should be a pure P-controller.
/// 1. No Dterm is required because the inner velocity loop inherently acts as the "D" (derivative) term for the outer position loop.
///    (Adding a D-term to the outer loop would mean calculating the derivative of velocity (acceleration), which would introduce severe sensor noise).
/// 2. No Iterm is required because steady-state errors (like wind blowing the aircraft sideways) are handled by the inner speed loop's Iterm.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct XyPositionDualRingPid {
    position_pid: PidControllerf32,
    speed_pid: PidControllerf32,
    max_speed_setpoint_mps: f32,
}

impl Default for XyPositionDualRingPid {
    fn default() -> Self {
        Self::new()
    }
}

impl XyPositionDualRingPid {
    pub const fn new() -> Self {
        Self {
            position_pid: PidControllerf32::new(1.0),
            // Inner loop: P for reactivity, I for wind correction, D for braking
            speed_pid: PidControllerf32::with_gains(PidGainsf32 { kp: 0.15, ki: 0.02, kd: 0.01, kk: 0.0, ks: 0.0 }),
            max_speed_setpoint_mps: 100.0,
        }
    }

    pub fn set_setpoint(&mut self, position_setpoint: f32) {
        self.position_pid.set_setpoint(position_setpoint);
    }

    /// Computes the required acceleration force corrections along a single linear coordinate axis.
    pub fn update(&mut self, current_position: f32, current_velocity: f32, delta_t: f32) -> f32 {
        // Outer Loop Stage: Position Error maps directly to a target speed profile
        let speed_setpoint = self
            .position_pid
            .update(current_position, delta_t)
            .clamp(-self.max_speed_setpoint_mps, self.max_speed_setpoint_mps);

        // Inner Loop Stage: Speed Error maps directly to an output tracking Force
        self.speed_pid.set_setpoint(speed_setpoint);
        self.speed_pid.update(current_velocity, delta_t)
    }

    pub fn reset(&mut self) {
        self.position_pid.reset();
        self.speed_pid.reset();
    }
}
