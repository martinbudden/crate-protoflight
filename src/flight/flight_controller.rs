use crate::flight::{
    rx_message::RcControls,
    vehicle_controller::{VehicleControlInitializing, VehicleController},
    {FlightModeConfig, VehicleControl},
};

use motor_mixers::{MotorMixer, MotorMixerCommon};
use pidsk_controller::{PidControllerf32, PidGainsf32};
use radio_controllers::RcModes;
use signal_filters::{Pt1FilterVector4df32, Pt1Filterf32, UpdateFilter};
use vqm::{Quaternionf32, Vector3df32, Vector4df32};

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlightController {
    vehicle_controller: VehicleController,
    angle_mode_calculation_state: AngleModeCalculationState,
    pub pids: [PidControllerf32; Self::PID_COUNT],
    // Copy of pid gains, so that gains can be adjusted by anti-gravity and then set back to their original values
    pub pid_gains: [PidGainsf32; Self::PID_COUNT],
    dterm_filters_0: [Pt1Filterf32; Self::PID_COUNT],
    dterm_filters_1: [Pt1Filterf32; Self::PID_COUNT],
    motor_commands_filter: Pt1FilterVector4df32,
    motor_commands_throttle: f32,
    flight_mode_config: FlightModeConfig,

    stabilization_mode: u8,
    use_angle_mode: bool,
    ground_mode: bool,
    use_level_race_mode: bool,

    crash_detected: bool,
    yaw_spin_recovery: bool,
    crash_flip_mode_active: bool,

    take_off_count_start: u32,
    take_off_throttle_threshold: f32,
    take_off_tick_threshold: u32,

    controls_tick_count: u32,
    blackbox_active: bool,

    max_roll_angle_degrees: f32,
    max_roll_rate_dps: f32,
    max_pitch_angle_degrees: f32,
    max_pitch_rate_dps: f32,
    tpa: f32,                   // Throttle PID Attenuation, reduces DTerm for large throttle values
    dmax_multipliers: [f32; 2], // used even if dmax feature not used
}

impl FlightController {
    pub const ROLL_RATE_DPS: usize = 0;
    pub const PITCH_RATE_DPS: usize = 1;
    pub const YAW_RATE_DPS: usize = 2;
    pub const ROLL_ANGLE_DEGREES: usize = 3;
    pub const PITCH_ANGLE_DEGREES: usize = 4;
    pub const PID_COUNT: usize = 5;

    pub const FD_ROLL: usize = 0;
    pub const FD_PITCH: usize = 1;
    //const FD_YAW: usize = 2;
    //const RPY_AXIS_COUNT: usize = 3;
}
impl Default for FlightController {
    fn default() -> Self {
        Self::new()
    }
}

impl FlightController {
    pub const fn new() -> Self {
        Self {
            vehicle_controller: VehicleController::new(),
            angle_mode_calculation_state: AngleModeCalculationState::new(),
            pids: [PidControllerf32::new(1.0); Self::PID_COUNT],
            pid_gains: [PidGainsf32::new(1.0, 0.0, 0.0, 0.0, 0.0); Self::PID_COUNT],
            dterm_filters_0: [Pt1Filterf32::new(); Self::PID_COUNT],
            dterm_filters_1: [Pt1Filterf32::new(); Self::PID_COUNT],
            motor_commands_filter: Pt1FilterVector4df32::new(),
            motor_commands_throttle: 0.0,
            flight_mode_config: FlightModeConfig::new(),

            stabilization_mode: 0,
            use_angle_mode: false,
            ground_mode: true,
            use_level_race_mode: false,

            crash_detected: false,
            yaw_spin_recovery: false,
            crash_flip_mode_active: false,

            take_off_count_start: 0,
            take_off_throttle_threshold: 0.1,
            take_off_tick_threshold: 10,

            controls_tick_count: 0,
            blackbox_active: false,

            max_roll_angle_degrees: 60.0,
            max_roll_rate_dps: 1000.0,
            max_pitch_angle_degrees: 60.0,
            max_pitch_rate_dps: 1000.0,
            tpa: 1.0, // Throttle PID Attenuation, reduces DTerm for large throttle values
            dmax_multipliers: [1.0, 1.0],
        }
    }
}

impl VehicleControl for FlightController {
    fn vehicle_controller(&self) -> &VehicleController {
        &self.vehicle_controller
    }
    fn vehicle_controller_mut(&mut self) -> &mut VehicleController {
        &mut self.vehicle_controller
    }
    // NOTE: CALLED FROM WITHIN THE AHRS TASK
    // It is typically called at frequency of between 1000Hz and 8000Hz, so it has to be FAST.
    //
    // The FlightController uses the NED (North-East-Down) coordinate convention.
    // gyro_rps, acc, and orientation come from the AHRS and use the ENU (East-North-Up) coordinate convention.
    fn calculate_motor_commands(
        &mut self,
        gyro_rps: Vector3df32,
        orientation: Quaternionf32,
        delta_t: f32,
        controls: RcControls,
    ) -> (Vector4df32, bool) {
        let mut setpoints_updated: bool = false;
        if controls.tick_count > self.controls_tick_count {
            // we have a new set of values from the receiver, so update the setpoints.
            self.controls_tick_count = controls.tick_count;
            self.update_setpoints(controls);
            setpoints_updated = true;
        }

        if self.crash_flip_mode_active {
            return (self.apply_crash_flip_to_motors(gyro_rps, delta_t), true);
        }

        if self.yaw_spin_recovery {
            return (self.recover_from_yaw_spin(gyro_rps, delta_t), true);
        }

        self.calculate_dmax_multipliers();

        if self.use_angle_mode {
            self.update_rate_setpoints_for_angle_mode(orientation, delta_t);
        }
        // Use the PIDs to calculate the outputs for each axis.
        // Note that the delta-values (ie the DTerms) are filtered:
        // this is because they are especially noisy, being the derivative of a noisy value.

        // The output from the PIDs is filtered.
        // This smooths the output, but also accumulates the output in the filter,
        // so the values influence the output even when `output_to_motors` is not called.

        //
        // Roll axis.
        // Note that the iterm and dterm are calculated outside the PID controller.
        // This allows dterm filtering and dynamic adjustment of the iterm and dterm (iterm relaxation and dmax).
        //
        let roll_rate_dps = Self::roll_rate_ned_dps(gyro_rps);
        let roll_iterm_error = self.calculate_iterm_error(Self::ROLL_RATE_DPS, roll_rate_dps);
        // filter the Dterm twice
        let roll_dterm = (roll_rate_dps - self.pids[Self::ROLL_RATE_DPS].previous_measurement())
            .filter_using(&mut self.dterm_filters_0[Self::ROLL_RATE_DPS])
            .filter_using(&mut self.dterm_filters_1[Self::ROLL_RATE_DPS])
            * self.dmax_multipliers[Self::ROLL_RATE_DPS]
            * self.tpa;

        let motor_command_roll_dps =
            self.pids[Self::ROLL_RATE_DPS].update_delta_iterm(roll_rate_dps, roll_dterm, roll_iterm_error, delta_t);
        //.filter_using(&mut self.motor_command_filters[Self::FD_ROLL]);

        //
        // Pitch axis
        // Note that the iterm and dterm are calculated outside the PID controller.
        // This allows dterm filtering and dynamic adjustment of the iterm and dterm (iterm relaxation and dmax).
        //
        let pitch_rate_dps = Self::pitch_rate_ned_dps(gyro_rps);
        let pitch_iterm_error = self.calculate_iterm_error(Self::PITCH_RATE_DPS, pitch_rate_dps);
        // filter the DTerm twice
        let pitch_dterm = (pitch_rate_dps - self.pids[Self::PITCH_RATE_DPS].previous_measurement())
            .filter_using(&mut self.dterm_filters_0[Self::PITCH_RATE_DPS])
            .filter_using(&mut self.dterm_filters_1[Self::PITCH_RATE_DPS])
            * self.dmax_multipliers[Self::PITCH_RATE_DPS]
            * self.tpa;

        let motor_command_pitch_dps =
            self.pids[Self::PITCH_RATE_DPS].update_delta_iterm(pitch_rate_dps, pitch_dterm, pitch_iterm_error, delta_t);
        //.filter_using(&mut self.motor_command_filters[FD_PITCH]);

        //
        // Yaw axis
        // Dterm is zero for yaw_rate, so call adjust_using_spi() with no Dterm filtering, no TPA, no dmax, no iterm relaxation, and no kterm (kick).
        //
        let yaw_rate_dps = Self::yaw_rate_ned_dps(gyro_rps);
        let motor_command_yaw_dps = self.pids[Self::YAW_RATE_DPS].update(yaw_rate_dps, delta_t);
        //.filter_using(&mut self.motor_command_filters[FD_YAW]);

        // Throttle.
        let motor_commands = Vector4df32 {
            x: motor_command_roll_dps,
            y: motor_command_pitch_dps,
            z: motor_command_yaw_dps,
            t: self.motor_commands_throttle,
        };

        (motor_commands.filter_using(&mut self.motor_commands_filter), setpoints_updated)
    }
}

#[allow(unused)]
impl FlightController {
    #[inline]
    pub fn roll_rate_ned_dps(gyro_enu_rps: Vector3df32) -> f32 {
        gyro_enu_rps.y.to_degrees()
    }

    #[inline]
    pub fn pitch_rate_ned_dps(gyro_enu_rps: Vector3df32) -> f32 {
        gyro_enu_rps.x.to_degrees()
    }

    #[inline]
    pub fn yaw_rate_ned_dps(gyro_enu_rps: Vector3df32) -> f32 {
        gyro_enu_rps.z.to_degrees()
    }

    // static inline float roll_sin_angle_ned(const Quaternion& orientation) { return orientation.sin_pitch_clipped(); } // sin(x-180) = -sin(x)
    // static inline float roll_cos_angle_ned(const Quaternion& orientation) { return orientation.cos_pitch(); }

    #[inline]
    pub fn roll_sin_angle_ned(orientation: Quaternionf32) -> f32 {
        orientation.sin_pitch_clipped()
    }

    #[inline]
    pub fn roll_cos_angle_ned(orientation: Quaternionf32) -> f32 {
        orientation.cos_pitch()
    }

    #[inline]
    pub fn roll_angle_degrees_ned(orientation: Quaternionf32) -> f32 {
        orientation.calculate_pitch_degrees()
    }

    #[inline]
    pub fn pitch_sin_angle_ned(orientation: Quaternionf32) -> f32 {
        orientation.sin_roll_clipped()
    }

    #[inline]
    pub fn pitch_cos_angle_ned(orientation: Quaternionf32) -> f32 {
        orientation.cos_roll()
    }

    #[inline]
    pub fn pitch_angle_degrees_ned(orientation: Quaternionf32) -> f32 {
        orientation.calculate_roll_degrees()
    }
}

#[allow(unused)]
impl FlightController {
    pub fn motors_switch_off(&mut self, motor_mixer: &mut MotorMixerCommon) {
        motor_mixer.motors_switch_off();
        //self.ground_mode = true;
        self.switch_pid_integration_off();
    }

    pub fn motors_switch_on(&mut self, motor_mixer: &mut MotorMixerCommon) {
        // don't allow motors to be switched on if the sensor fusion has not initialized
        if !self.vehicle_controller().sensor_fusion_filter_is_initializing() {
            motor_mixer.motors_switch_on();
            // reset the PID integral values when we switch the motors on
            self.switch_pid_integration_on();
        }
    }
    pub fn switch_pid_integration_on(&mut self) {
        for pid in &mut self.pids {
            pid.switch_integration_on();
        }
    }

    pub fn switch_pid_integration_off(&mut self) {
        for pid in &mut self.pids {
            pid.switch_integration_off();
        }
    }

    pub fn set_stabilization_mode(&mut self, stabilization_mode: u8) {
        if stabilization_mode == self.stabilization_mode {
            return;
        }
        self.stabilization_mode = stabilization_mode;
        // reset the PID integral values when we change control mode
        for pid in &mut self.pids {
            pid.reset_integral();
        }
    }

    #[allow(clippy::unused_self)]
    pub fn recover_from_yaw_spin(&mut self, _gyro_rps: Vector3df32, _delta_t: f32) -> Vector4df32 {
        Vector4df32::default()
    }

    #[inline]
    pub fn calculate_dmax_multipliers(&mut self) {
        self.dmax_multipliers[Self::FD_ROLL] = 1.0;
        self.dmax_multipliers[Self::FD_PITCH] = 1.0;
    }

    #[inline]
    pub fn calculate_iterm_error(&self, axis: usize, measurement: f32) -> f32 {
        let setpoint = self.pids[axis].setpoint();
        // iterm_error is just `setpoint - measurement`, if there is no iterm relax
        setpoint - measurement
    }

    #[allow(clippy::unused_self)]
    pub fn apply_crash_flip_to_motors(&mut self, _gyro_rps: Vector3df32, _delta_t: f32) -> Vector4df32 {
        Vector4df32::default()
    }

    pub fn update_setpoints(&mut self, controls: RcControls) {
        //detect_crash_or_spin();

        self.set_stabilization_mode(controls.stabilization_mode);

        // output throttle may be changed by spin recovery
        self.motor_commands_throttle = controls.throttle_stick;

        /*if controls.failsafe == FAILSAFE_ON || self.crash_detected || self.yaw_spin_recovery || self.crash_flip_mode_active {
            clear_dynamic_pid_adjustments();
        } else {
            apply_dynamic_pid_adjustments_on_throttle_change(controls.throttle_stick, controls.tick_count, debug);
        }*/

        //
        // Roll axis
        //
        // Pushing the ROLL stick to the right gives a positive value of roll_stick and we want this to be left side up.
        // For NED left side up is positive roll, so sign of setpoint is same sign as roll_stick.
        // So sign of _roll_stick is left unchanged.
        if !self.use_angle_mode {
            self.pids[Self::ROLL_RATE_DPS].set_setpoint(controls.roll_stick_dps);
        }
        self.pids[Self::ROLL_ANGLE_DEGREES].set_setpoint(controls.roll_stick_degrees);
        //
        // Pitch axis
        //
        // Pushing the  PITCH stick forward gives a positive value of _pitch_stick and we want this to be nose down.
        // For NED nose down is negative pitch, so sign of setpoint is opposite sign as _pitch_stick.
        // So sign of _pitch_stick is negated.
        if !self.use_angle_mode {
            self.pids[Self::PITCH_RATE_DPS].set_setpoint(-controls.pitch_stick_dps);
        }
        self.pids[Self::PITCH_ANGLE_DEGREES].set_setpoint(-controls.pitch_stick_degrees);

        //
        // Yaw axis
        //
        // Pushing the YAW stick to the right gives a positive value of _yaw_stick and we want this to be nose right.
        // For NED nose left is positive yaw, so sign of setpoint is same as sign of _yaw_stick.
        // So sign of _yaw_stick is left unchanged.
        self.pids[Self::YAW_RATE_DPS].set_setpoint(controls.yaw_stick_dps);

        //
        // Modes
        //
        // When in ground mode, the PID I-terms are set to zero to avoid integral windup on the ground
        if self.ground_mode {
            // exit ground mode if the throttle has been above _take_off_throttle_threshold for _take_off_tick_threshold ticks
            if self.motor_commands_throttle < self.take_off_throttle_threshold {
                self.take_off_count_start = 0;
            } else {
                let tick_count = controls.tick_count;
                if self.take_off_count_start == 0 {
                    self.take_off_count_start = tick_count;
                }
                if tick_count - self.take_off_count_start > self.take_off_tick_threshold {
                    self.ground_mode = false;
                    // we've exited ground mode, so we can turn on PID integration
                    self.switch_pid_integration_on();
                }
            }
        }
        // Angle Mode is used if the control_mode is set to angle mode, or failsafe is on.
        // Angle Mode is prevented when in Ground Mode, so the aircraft doesn't try and self-level while it is still on the ground.
        // This value is cached here, to avoid evaluating a reasonably complex condition in update_outputs_using_pids()
        self.use_angle_mode = (self.stabilization_mode >= RcModes::STABILIZATION_MODE_ANGLE) && !self.ground_mode;
        self.use_level_race_mode = (self.stabilization_mode == RcModes::STABILIZATION_MODE_LEVEL_RACE)
            || (self.flight_mode_config.level_race_mode != 0);
    }
}

impl FlightController {
    /// NOTE: CALLED FROM WITHIN THE AHRS TASK.
    ///
    /// In angle mode, the roll and pitch angles are used to set the setpoints for the rollRate and pitchRate PIDs.
    /// Level Race Mode (aka NFE(Not Fast Enough) mode) is equivalent to angle mode on roll and acro mode on pitch.
    #[allow(clippy::unused_self)]
    fn update_rate_setpoints_for_angle_mode(&mut self, _orientation: Quaternionf32, _delta_t: f32) {
        //self.angle_mode_calculation_state.update(&mut self, orientation, delta_t)
    }
}

/// State machine to calculate setpoints for angle mode.
/// Calculates one axis per iteration.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum AngleModeCalculationState {
    #[default]
    CalculateRoll,
    //CalculatePitch,
}

impl AngleModeCalculationState {
    pub const fn new() -> Self {
        Self::CalculateRoll
    }
}

/*impl AngleModeCalculationState {
    /// Perform one step of the state machine.
    fn update(&mut self, fc: &mut FlightController, orientation: Quaternionf32, delta_t: f32) {
        match core::mem::take(self) {
            AngleModeCalculationState::CalculateRoll => {
                let roll_angle_degrees = FlightController::roll_angle_degrees_ned(orientation);
                //let roll_angle_delta = fc.dterm_filters_0[ROLL_ANGLE_DEGREES]
                //    .update(roll_angle_degrees - fc.pids[ROLL_ANGLE_DEGREES].previous_measurement());
                let roll_angle_delta = (roll_angle_degrees - fc.pids[ROLL_ANGLE_DEGREES].previous_measurement())
                    .filter_using(&mut fc.dterm_filters_0[ROLL_ANGLE_DEGREES]);

                // calculate roll rate setpoint in degrees, range is [-_max_roll_angle_degrees, _max_roll_angle_degrees], typically [-60, 60]
                //let roll_rate_setpoint_degrees =
                //    fc.pids[ROLL_ANGLE_DEGREES].update_delta(roll_angle_degrees, roll_angle_delta, delta_t);
                let roll_rate_setpoint_degrees =
                    roll_angle_degrees.adjust_using_d(&mut fc.pids[ROLL_ANGLE_DEGREES], roll_angle_delta, delta_t);

                // convert to value in range [-1.0, 1.0] to be used for the ROLL_RATE_DPS setpoint
                let roll_rate_setpoint_dps =
                    (roll_rate_setpoint_degrees / fc.max_roll_angle_degrees).clamp(-1.0, 1.0) * fc.max_roll_rate_dps;

                fc.pids[ROLL_RATE_DPS].set_setpoint(roll_rate_setpoint_dps);

                // in level race mode we use angle mode on roll, acro mode on pitch, so keep state as CalculateRoll
                if fc.stabilization_mode != VehicleControls::MODE_LEVEL_RACE {
                    *self = AngleModeCalculationState::CalculatePitch;
                }
            }

            AngleModeCalculationState::CalculatePitch => {
                let pitch_angle_degrees = FlightController::pitch_angle_degrees_ned(orientation);
                let pitch_angle_delta = (pitch_angle_degrees - fc.pids[PITCH_ANGLE_DEGREES].previous_measurement())
                    .filter_using(&mut fc.dterm_filters_0[PITCH_ANGLE_DEGREES]);

                // calculate pitch rate setpoint in degrees, range is [-_max_pitch_angle_degrees, _max_pitch_angle_degrees], typically [-60, 60]
                let pitch_rate_setpoint_degrees =
                    pitch_angle_degrees.adjust_using_d(&mut fc.pids[PITCH_ANGLE_DEGREES], pitch_angle_delta, delta_t);

                // convert to value in range [-1.0, 1.0] to be used for the PITCH_RATE_DPS setpoint
                let pitch_rate_setpoint_dps =
                    (pitch_rate_setpoint_degrees / fc.max_pitch_angle_degrees).clamp(-1.0, 1.0) * fc.max_pitch_rate_dps;

                fc.pids[PITCH_RATE_DPS].set_setpoint(pitch_rate_setpoint_dps);

                *self = AngleModeCalculationState::CalculateRoll;
            }
        }
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused)]
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    #[allow(unused)]
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<FlightController>();
    }
    #[test]
    fn test_new() {
        let flight_controller = FlightController::new();
        assert_eq!(0, flight_controller.stabilization_mode);
    }
}
