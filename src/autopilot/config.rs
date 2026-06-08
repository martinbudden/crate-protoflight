#![allow(unused)]
use crate::flight::PidConfig;
#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

// RX Loss Policy: What to do when radio signal is lost during autopilot
pub struct AutopilotRxLoss {}
impl AutopilotRxLoss {
    pub const DISABLE: u8 = 0; // Disable autopilot, use standard failsafe
    pub const CONTINUE: u8 = 1; // Continue mission (if GPS valid)
    pub const LAND: u8 = 2; // Land at current position
}

pub struct AutopilotYawMode {}
impl AutopilotYawMode {
    pub const VELOCITY: u8 = 0; // Multirotor: point nose in velocity direction
    pub const BEARING: u8 = 1; // Multirotor: point nose at waypoint
    pub const HYBRID: u8 = 2; // Multirotor: blend based on distance
    pub const FIXED: u8 = 3; // Multirotor: no yaw control
    pub const DAMPENER: u8 = 4; // Wing: yaw rate damper (coordinated turns)
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AutopilotConfig {
    pub landing_altitude_m: u8, // altitude below which landing behaviors can change, metres
    pub hover_throttle: u16,    // value used at the start of a rescue or position hold
    pub throttle_min: u16,
    pub throttle_max: u16,
    pub altitude_pid: PidConfig,
    pub position_pid: PidConfig,
    pub position_i_i: u8,
    pub position_a: u8,
    pub position_cutoff: u8,
    pub max_angle: u8,

    // Velocity-based position control with drag compensation
    pub velocity_control_enable: u8, // 0=position→angle, 1=position→velocity→angle (default 1)
    pub velocity_pid: PidConfig,
    pub velocity_drag_coeff: u16, // drag coefficient, scaled by 10000 (default 50 = 0.0050)
    pub max_velocity_cmps: u16,   // cm/s, maximum velocity setpoint (default 1000 = 10 m/s)

    // Waypoint navigation parameters
    pub waypoint_arrival_radius_cm: u16, // cm, arrival detection radius (default 500)
    pub waypoint_hold_radius_cm: u16,    // cm, hold pattern radius (default 200)
    pub stick_deadband: u16,             // RC units (0-500), deadband for pilot override (default 50)
    pub throttle_deadband: u16,          // RC units (0-500), throttle override deadband (default 50)

    // Yaw control parameters
    pub yaw_mode: u8,
    pub yaw_pid: PidConfig,             // scaled by 100 (e.g., 50 = 0.5)
    pub max_yaw_rate_dps: u16,          // deg/s, maximum yaw rate
    pub min_forward_velocity_cmps: u16, // cm/s, minimum forward velocity: G_pS course reliability (multirotor) / stall prevention (wing)

    // Velocity buildup (acceleration from stationary)
    pub velocity_buildup_max_pitch: u8, // degrees, max pitch bias for acceleration (default 8)

    // Turn rate and holding patterns
    pub max_turn_rate_dps: u8, // deg/s, maximum turn rate (default 3 for rate-1 turn, configurable 1-90)
    pub hold_orbit_radius_cm: u16, // cm, radius for orbit holding pattern (default 1000 = 10m)
    pub hold_figure8_width_cm: u16, // cm, width of figure-8 pattern (default 2000 = 20m)

    // Landing sequence
    pub landing_descent_rate: u8, // cm/s, descent rate during landing (default 50 = 0.5 m/s)
    pub landing_detection_time: u8, // deciseconds, time below threshold for touchdown detection (default 10 = 1s)
    pub landing_spiral_enable: u8, // 0=straight descent, 1=spiral (avoid vortex ring state, default 1)
    pub landing_spiral_radius: u16, // cm, radius of spiral pattern (default 200 = 2m)
    pub landing_spiral_rate: u8,  // deg/s, rotation rate (default 10 = 10 deg/s)
    pub landing_velocity_threshold: u8, // cm/s, max velocity for touchdown (default 50 = 0.5 m/s)
    pub landing_throttle_threshold: u16, // RC units, max throttle deviation for touchdown (default 100)

    // L1 Nonlinear Guidance parameters
    pub l1_enable: u8,                 // 0=direct targeting, 1=L1 guidance (default 1)
    pub l1_period: u16,                // L1 damping period, deciseconds (default 20 = 2.0s)
    pub l1_min_lookahead: u16,         // Minimum lookahead distance, cm (default 1000 = 10m)
    pub l1_max_lookahead: u16,         // Maximum lookahead distance, cm (default 10000 = 100m)
    pub l1_max_cross_track_error: u16, // Max cross-track error before fallback, cm (default 10000 = 100m)
    pub l1_turn_rate: u8, // deg/s, max turn rate for arc transitions between waypoints (default 8, 0=disabled)

    // Vertical track: climb to min altitude, then follow glide slope to waypoint
    pub min_nav_altitude_m: u8, // metres AGL, minimum altitude before horizontal navigation (default 5)

    // Safety: RX loss policy
    pub rx_loss_policy: u8,

    // Safety: geofence (max distance from home)
    pub max_distance_from_home_m: u8, // meters, 0 = disabled (default 0)
    pub geofence_action: u8,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for AutopilotConfig {}

impl AutopilotConfig {
    pub const fn new() -> Self {
        Self {
            landing_altitude_m: 4,
            hover_throttle: 1275,
            throttle_min: 1100,
            throttle_max: 1900,
            altitude_pid: PidConfig { kp: 15, ki: 15, kd: 15, ks: 0, kk: 15 },
            position_pid: PidConfig { kp: 30, ki: 30, kd: 30, ks: 0, kk: 30 },
            position_i_i: 30,
            position_a: 30,
            position_cutoff: 30,
            max_angle: 50,

            velocity_control_enable: 1,
            velocity_pid: PidConfig { kp: 50, ki: 10, kd: 5, ks: 0, kk: 0 },
            velocity_drag_coeff: 50, // 0.0050 drag coefficient
            max_velocity_cmps: 1000, // 10 m/s max velocity

            waypoint_arrival_radius_cm: 500, // 5m for FLYOVER/FLYBY
            waypoint_hold_radius_cm: 200,    // 2m for HOLD/LAND
            stick_deadband: 50,              // RC units
            throttle_deadband: 50,           // RC units

            yaw_mode: AutopilotYawMode::VELOCITY,
            yaw_pid: PidConfig { kp: 0, ki: 0, kd: 0, ks: 0, kk: 0 },
            max_yaw_rate_dps: 0,
            min_forward_velocity_cmps: 0,

            velocity_buildup_max_pitch: 0,

            max_turn_rate_dps: 0,
            hold_orbit_radius_cm: 0,
            hold_figure8_width_cm: 0,

            landing_descent_rate: 0,
            landing_detection_time: 0,
            landing_spiral_enable: 0,
            landing_spiral_radius: 0,
            landing_spiral_rate: 0,
            landing_velocity_threshold: 0,
            landing_throttle_threshold: 0,

            l1_enable: 0,
            l1_period: 0,
            l1_min_lookahead: 0,
            l1_max_lookahead: 0,
            l1_max_cross_track_error: 0,
            l1_turn_rate: 0,

            min_nav_altitude_m: 0,

            rx_loss_policy: 0,

            max_distance_from_home_m: 0,
            geofence_action: 0,
        }
    }
}

impl Default for AutopilotConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PositionHoldConfig {
    pub deadband: u8,
    pub position_source: u8,            // Position source selection
    pub optical_flow_quality_min: u8,   // Minimum optical flow quality threshold
    pub optical_flow_max_range_cm: u16, // Maximum altitude for optical flow (cm)
}

impl PositionHoldConfig {
    pub const SOURCE_AUTO: u8 = 0;
    pub const SOURCE_GPS_ONLY: u8 = 1;
    pub const SOURCE_OPTICAL_FLOW_ONLY: u8 = 2;

    pub const fn new() -> Self {
        Self { deadband: 0, position_source: 0, optical_flow_quality_min: 0, optical_flow_max_range_cm: 0 }
    }
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for PositionHoldConfig {}

impl Default for PositionHoldConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<AutopilotConfig>();
        is_full::<PositionHoldConfig>();
        #[cfg(feature = "serde")]
        is_config::<AutopilotConfig>();
        #[cfg(feature = "serde")]
        is_config::<PositionHoldConfig>();
    }
    #[test]
    fn test_new() {
        let config = AutopilotConfig::new();
        assert_eq!(4, config.landing_altitude_m);
    }
}
