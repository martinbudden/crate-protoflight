use super::{
    AntiGravity, DMaxConfig, FlightModeConfig, ItermRelaxConfig, Tpa, VehicleControl,
    rx_message::RcControls,
    vehicle_controller::{VehicleControlInitializing, VehicleController},
};

#[cfg(feature = "debug")]
use crate::tasks::{DebugMode, GLOBAL_DEBUG};

use motor_mixers::MotorMixer;
use pidsk_controller::{PdControllerf32, PidskControllerf32, PidskGainsf32};
use radio_controllers::{RcMode, RxLinkStatus};
use signal_filters::{Pt1FilterVector4f32, Pt1Filterf32, SignalFilter, UpdateFilter};
use simple_bitset::BitSet64;
use vqm::{Quaternionf32, Vector3f32, Vector4f32};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd)]
enum FlightStabilizationMode {
    #[default]
    Rate = 0,
    Angle = 1,
    #[allow(unused)]
    Horizon = 2,
    LevelRace = 3,
}

impl FlightStabilizationMode {
    /// Forgiving conversion from `u8` to `FlightStabilizationMode`, converts invalid values to default.
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Rate,

            // Flight mode flags
            1 => Self::Angle,
            2 => Self::Horizon,
            3 => Self::LevelRace,
            _ => Self::default(),
        }
    }
}

impl TryFrom<u8> for FlightStabilizationMode {
    type Error = ();

    /// Validating conversion from `u8` to `FlightStabilizationMode`. Invalid values return error.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let default = Self::default();
        if value == default as u8 {
            Ok(default)
        } else {
            let ret = Self::from_u8(value);
            if ret == default { Err(()) } else { Ok(ret) }
        }
    }
}

/// Properties common to the roll and pitch axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FcTilt {
    pub rate_pid: PidskControllerf32,
    /// Value of gains as configured. Used to restore gains after dynamic pid changes.
    pub rate_gains: PidskGainsf32,
    rate_dterm_filters: [Pt1Filterf32; 2],
    max_rate_dps: f32,
    dmax_multiplier: f32,
    dmax_percent: f32,
    dmax_range_filter: Pt1Filterf32,
    dmax_lpf: Pt1Filterf32,
    iterm_relax_filter: Pt1Filterf32,
    setpoint_filter: Pt1Filterf32,

    pub angle_pid: PdControllerf32,
    angle_dterm_filter: Pt1Filterf32,
    max_angle_degrees: f32,
}

impl Default for FcTilt {
    fn default() -> Self {
        Self::new()
    }
}

impl FcTilt {
    pub const fn new() -> Self {
        Self {
            rate_pid: PidskControllerf32::new(),
            rate_gains: PidskGainsf32::new(),
            rate_dterm_filters: [Pt1Filterf32::new(); 2],
            max_rate_dps: 1000.0,
            dmax_multiplier: 1.0,
            dmax_percent: 0.5,
            dmax_range_filter: Pt1Filterf32::new(),
            dmax_lpf: Pt1Filterf32::new(),
            iterm_relax_filter: Pt1Filterf32::new(),
            setpoint_filter: Pt1Filterf32::new(),

            angle_pid: PdControllerf32::new(),
            angle_dterm_filter: Pt1Filterf32::new(),
            max_angle_degrees: 60.0,
        }
    }
}

impl FcTilt {
    fn calculate_dmax_multipliers(&mut self, delta_t: f32, dmax_gyro_gain: f32, dmax_setpoint_gain: f32) {
        //TODO: check using PID error in DMAX, surely this is too easy
        let gyro_delta_d = delta_t * self.rate_pid.error().d;
        let gyro_factor = (self.dmax_range_filter.update(gyro_delta_d)).abs() * dmax_gyro_gain;
        let setpoint_factor = (self.rate_pid.setpoint_delta()).abs() * dmax_setpoint_gain;
        let boost = gyro_factor.max(setpoint_factor);
        // boost starts at zero, and by 1.0 we get Dmax, but it can exceed 1.0
        self.dmax_multiplier += (self.dmax_percent - 1.0) * boost;
        self.dmax_multiplier = self.dmax_lpf.update(self.dmax_multiplier);
        // limit the multiplier to dmax_percent
        self.dmax_multiplier = (self.dmax_multiplier).min(self.dmax_percent);
    }

    pub fn calculate_rate_iterm_relax_factor(&mut self, setpoint: f32, setpoint_threshold_dps: f32) -> f32 {
        let setpoint_lp = self.setpoint_filter.update(setpoint);
        let setpoint_hp = (setpoint - setpoint_lp).abs();
        (1.0 - setpoint_hp / setpoint_threshold_dps).max(0.0)
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlightController {
    vehicle_controller: VehicleController,
    angle_mode_calculation_state: AngleModeCalculationState,

    pub roll: FcTilt,
    pub pitch: FcTilt,
    pub yaw_rate_pid: PidskControllerf32,
    /// Value of gains as configured. Used to restore gains after dynamic pid changes.
    pub yaw_rate_gains: PidskGainsf32,

    // Copy of pid gains, so that gains can be adjusted by anti-gravity and then set back to their original values
    //pub pid_roll_rate_gains: PidskGainsf32,
    //pub pid_pitch_rate_gains: PidskGainsf32,
    motor_commands_filter: Pt1FilterVector4f32,
    motor_commands_throttle: f32,
    flight_mode_config: FlightModeConfig,

    stabilization_mode: FlightStabilizationMode,
    use_angle_mode: bool,
    ground_mode: bool,
    use_level_race_mode: bool,
    use_iterm_relax: bool,

    crash_detected: bool,
    yaw_spin_recovery: bool,
    crash_flip_mode_active: bool,

    take_off_count_start: u32,
    take_off_throttle_threshold: f32,
    take_off_tick_threshold: u32,

    controls_tick_count: u32,
    blackbox_active: bool,

    tpa: Tpa,
    throttle_previous: f32,
    anti_gravity: AntiGravity,
    dmax_gyro_gain: f32,
    dmax_setpoint_gain: f32,
}

impl FlightController {
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

            roll: FcTilt::new(),
            pitch: FcTilt::new(),
            yaw_rate_pid: PidskControllerf32::new(),
            yaw_rate_gains: PidskGainsf32::new(),
            //pid_gains: [PidskGainsf32::new(); Self::PID_COUNT],
            /*pid_pitch_angle_gains: PidskGainsf32::new(),
            pid_roll_angle_gains: PidskGainsf32::new(),
            pid_yaw_rate_gains: PidskGainsf32::new(),
            pid_pitch_rate_gains: PidskGainsf32::new(),
            pid_roll_rate_gains: PidskGainsf32::new(),*/
            motor_commands_filter: Pt1FilterVector4f32::new(),
            motor_commands_throttle: 0.0,
            flight_mode_config: FlightModeConfig::new(),

            stabilization_mode: FlightStabilizationMode::Rate,
            ground_mode: true,
            use_angle_mode: false,
            use_level_race_mode: false,
            use_iterm_relax: false,

            crash_detected: false,
            yaw_spin_recovery: false,
            crash_flip_mode_active: false,

            take_off_count_start: 0,
            take_off_throttle_threshold: 0.1,
            take_off_tick_threshold: 10,

            controls_tick_count: 0,
            blackbox_active: false,

            tpa: Tpa::new(), // Throttle PID Attenuation, reduces DTerm for large throttle values
            throttle_previous: 0.0,
            anti_gravity: AntiGravity::new(),
            dmax_gyro_gain: 0.0,
            dmax_setpoint_gain: 0.0,
        }
    }
}

impl FlightController {
    #[allow(unused)]
    pub fn set_iterm_relax_config(&mut self, config: ItermRelaxConfig, delta_t: f32) {
        self.use_iterm_relax = config.relax;
        self.roll.iterm_relax_filter.set_cutoff_frequency(f32::from(config.relax_cutoff), delta_t);
        self.pitch.iterm_relax_filter.set_cutoff_frequency(f32::from(config.relax_cutoff), delta_t);
    }

    #[allow(unused)]
    pub fn set_dmax_config(&mut self, config: DMaxConfig) {
        const D_MAX_RANGE_HZ: f32 = 85.0; // lowpass input cutoff to peak D around propwash frequencies
        const D_MAX_LOWPASS_HZ: f32 = 35.0; // lowpass cutoff to smooth the boost effect.
        const D_MAX_GYRO_GAIN_FACTOR: f32 = 0.00008;
        const D_MAX_SETPOINT_GAIN_FACTOR: f32 = 0.00008; // same DMax gain with either rate of change source; not intended preserve legacy behavior

        self.dmax_gyro_gain = f32::from(config.gain) * D_MAX_GYRO_GAIN_FACTOR / D_MAX_LOWPASS_HZ;
        self.dmax_setpoint_gain = f32::from(config.advance) * D_MAX_SETPOINT_GAIN_FACTOR / D_MAX_LOWPASS_HZ;
    }
}

impl VehicleControl for FlightController {
    fn vehicle_controller(&self) -> &VehicleController {
        &self.vehicle_controller
    }
    fn vehicle_controller_mut(&mut self) -> &mut VehicleController {
        &mut self.vehicle_controller
    }

    // NOTE: CALLED FROM WITHIN THE GYRO/PID TASK
    // It is typically called at frequency of between 1000Hz and 8000Hz, so it has to be FAST.
    //
    // The FlightController uses the NED (North-East-Down) coordinate convention.
    // gyro_rps, acc, and orientation come from the AHRS and use the ENU (East-North-Up) coordinate convention.
    fn calculate_motor_commands(
        &mut self,
        gyro_rps: Vector3f32,
        orientation: Quaternionf32,
        delta_t: f32,
        controls: RcControls,
        rc_modes: BitSet64,
    ) -> (Vector4f32, bool) {
        let mut setpoints_updated: bool = false;

        if controls.tick_count > self.controls_tick_count {
            // we have a new set of values from the receiver, so update the setpoints.
            self.controls_tick_count = controls.tick_count;
            self.update_setpoints(controls, rc_modes);
            setpoints_updated = true;
            if controls.link_status != RxLinkStatus::Ok
                || self.crash_detected
                || self.yaw_spin_recovery
                || self.crash_flip_mode_active
            {
                self.clear_dynamic_pid_adjustments();
            } else {
                self.apply_dynamic_pid_adjustments_on_throttle_change(controls.throttle_stick, delta_t);
            }
        }

        if self.crash_flip_mode_active {
            setpoints_updated = true;
            return (self.apply_crash_flip_to_motors(gyro_rps, delta_t), setpoints_updated);
        }

        if self.yaw_spin_recovery {
            setpoints_updated = true;
            return (self.recover_from_yaw_spin(gyro_rps, delta_t), setpoints_updated);
        }

        if self.use_angle_mode {
            self.update_rate_setpoints_for_angle_mode(orientation, delta_t);
        }

        self.calculate_dmax_multipliers(delta_t);

        // Use the PIDs to calculate the outputs for each axis.
        // Note that the delta-values (ie the DTerms) are filtered:
        // this is because they are especially noisy, being the derivative of a noisy value.

        //
        // Roll axis.
        // Note that the iterm and dterm are calculated outside the PID controller.
        // This allows dterm filtering and dynamic adjustment of the iterm and dterm (iterm relaxation and dmax).
        //
        let roll_rate_dps = Self::roll_rate_ned_dps(gyro_rps);
        let roll_rate_iterm_error = self.calculate_roll_rate_iterm_error(roll_rate_dps);
        // filter the Dterm twice
        let roll_rate_dterm = (roll_rate_dps - self.roll.rate_pid.previous_measurement())
            .filter_using(&mut self.roll.rate_dterm_filters[0])
            .filter_using(&mut self.roll.rate_dterm_filters[1])
            * self.roll.dmax_multiplier
            * self.tpa.value;

        let motor_command_roll_dps =
            self.roll.rate_pid.update_delta_iterm(roll_rate_dps, roll_rate_dterm, roll_rate_iterm_error, delta_t);

        //
        // Pitch axis
        // Note that the iterm and dterm are calculated outside the PID controller.
        // This allows dterm filtering and dynamic adjustment of the iterm and dterm (iterm relaxation and dmax).
        //
        let pitch_rate_dps = Self::pitch_rate_ned_dps(gyro_rps);
        let pitch_rate_iterm_error = self.calculate_pitch_rate_iterm_error(pitch_rate_dps);
        // filter the DTerm twice
        let pitch_rate_dterm = (pitch_rate_dps - self.pitch.rate_pid.previous_measurement())
            .filter_using(&mut self.pitch.rate_dterm_filters[0])
            .filter_using(&mut self.pitch.rate_dterm_filters[1])
            * self.pitch.dmax_multiplier
            * self.tpa.value;

        let motor_command_pitch_dps =
            self.pitch.rate_pid.update_delta_iterm(pitch_rate_dps, pitch_rate_dterm, pitch_rate_iterm_error, delta_t);

        //
        // Yaw axis
        // Dterm is zero for yaw_rate, so call adjust_using_spi()
        // with no Dterm filtering, no TPA, no dmax_multiplier, no Iterm relaxation, and no Kterm (kick).
        //
        let yaw_rate_dps = Self::yaw_rate_ned_dps(gyro_rps);
        let motor_command_yaw_dps = self.yaw_rate_pid.update_spi(yaw_rate_dps, delta_t);

        // Put the motor commands into a vector, so that we can filter them all at once.
        let motor_commands = Vector4f32 {
            x: motor_command_roll_dps,
            y: motor_command_pitch_dps,
            z: motor_command_yaw_dps,
            t: self.motor_commands_throttle,
        };

        // Filter the motor commands.
        // This smooths the output, but also accumulates the output in the filter,
        // so the values influence the output even when `output_to_motors` is not called.
        (motor_commands.filter_using(&mut self.motor_commands_filter), setpoints_updated)
    }
}

/// Dynamic PID adjustments made when throttle changes:
/// These include:
/// Throttle PID Attenuation: lowers the roll rate and pitch rate P and D terms when the throttle is high
/// Anti-gravity: adjusts the roll rate and pitch rate P and I terms when the throttle is moving quickly
/// DMAX is not applied here, since it depends on the gyro value and so needs to be calculated in `update_outputs_using_pids()`.
impl FlightController {
    fn apply_dynamic_pid_adjustments_on_throttle_change(&mut self, throttle: f32, delta_t: f32) {
        let throttle_delta = (throttle - self.throttle_previous).abs();
        self.throttle_previous = throttle;
        let mut throttle_derivative = throttle_delta / delta_t;

        let throttle_reversed = 1.0 - throttle;
        throttle_derivative *= throttle_reversed * throttle_reversed;
        // generally focus on the low throttle period
        if throttle > self.throttle_previous {
            throttle_derivative *= throttle_reversed * 0.5;
            // when increasing throttle, focus even more on the low throttle range
        }
        // filtering suppresses peaks relative to troughs and prolongs the anti-gravity effects
        //throttleDerivative = _sh.anti_gravityThrottleFilter.filter(throttleDerivative);

        // ****
        // use anti-gravity to adjust the Iterms on roll and pitch
        // ****

        let i_term_accelerator = throttle_derivative * self.anti_gravity.i_gain * AntiGravity::KI;
        self.roll.rate_pid.set_ki(self.roll.rate_gains.ki + i_term_accelerator);
        self.pitch.rate_pid.set_ki(self.pitch.rate_gains.ki + i_term_accelerator);

        // ****
        // calculate the Throttle PID Attenuation (TPA)
        // TPA is applied here to the PTerms on roll and pitch, and is used as a multiplier
        // of the DTERM in update_outputs_using_pids.
        // ****

        // _TPA is 1.0 (ie no attenuation) if throttle_stick <= _tpaBreakpoint;
        self.tpa.value = 1.0 - self.tpa.multiplier * (throttle - self.tpa.breakpoint).max(0.0);
        #[cfg(feature = "debug")]
        GLOBAL_DEBUG.set_f32(DebugMode::Tpa, 0, self.tpa.value * 1000.0);

        // ****
        // use TPA and anti-gravity to adjust the PTerms on roll and pitch
        // ****

        // attenuate roll if setpoint greater than 50 DPS, half at 100 DPS
        let roll_attenuator = (self.roll.rate_pid.setpoint().abs() / 50.0).min(1.0);
        let roll_pterm_boost = 1.0 + (throttle_derivative * self.anti_gravity.p_gain / roll_attenuator);
        self.roll.rate_pid.set_kp(self.roll.rate_gains.kp * roll_pterm_boost * self.tpa.value);

        // attenuate pitch if setpoint greater than 50 DPS, half at 100 DPS
        let pitch_attenuator = (self.pitch.rate_pid.setpoint().abs() / 50.0).min(1.0);
        let pitch_pterm_boost = 1.0 + (throttle_derivative * self.anti_gravity.p_gain / pitch_attenuator);
        self.pitch.rate_pid.set_kp(self.pitch.rate_gains.kp * pitch_pterm_boost * self.tpa.value);
        #[cfg(feature = "debug")]
        GLOBAL_DEBUG.set_f32(DebugMode::AntiGravity, 3, pitch_pterm_boost * 1000.0);
    }

    fn clear_dynamic_pid_adjustments(&mut self) {
        self.tpa.value = 1.0;
        self.roll.rate_pid.set_kp(self.roll.rate_gains.kp);
        self.pitch.rate_pid.set_kp(self.pitch.rate_gains.kp);
    }
    /*
       #[inline]
       pub fn calculate_dmax_multipliers(&mut self) {
           self.roll.dmax_multiplier = 1.0;
           self.pitch.dmax_multiplier = 1.0;
       }
    */

    /// Calculate the dmax multipliers.
    /// These are multipliers that are applied to the roll and pitch axis `Dterms`.
    /// This means `Dterms` can be low in normal flight but are boosted to a higher value when required.
    /// They are boosted when the `Dterm` error is small and the setpoint change is also small.
    fn calculate_dmax_multipliers(&mut self, delta_t: f32) {
        //TODO: check using PID error in DMAX, surely this is too easy
        self.roll.calculate_dmax_multipliers(delta_t, self.dmax_gyro_gain, self.dmax_setpoint_gain);
        self.pitch.calculate_dmax_multipliers(delta_t, self.dmax_gyro_gain, self.dmax_setpoint_gain);

        #[cfg(feature = "debug")]
        if GLOBAL_DEBUG.mode() == DebugMode::DMax as u8 {
            //GLOBAL_DEBUG.set_f32(DebugMode::DMax, 0, gyro_factor * 100.0);
            //GLOBAL_DEBUG.set_f32(DebugMode::DMax, 1, setpoint_factor * 100.0);
            GLOBAL_DEBUG.set_f32(DebugMode::DMax, 2, self.roll.rate_pid.gains().kd * self.roll.dmax_multiplier * 100.0);
            GLOBAL_DEBUG.set_f32(
                DebugMode::DMax,
                3,
                self.pitch.rate_pid.gains().kd * self.pitch.dmax_multiplier * 100.0,
            );
        }
    }
    #[inline]
    pub fn calculate_roll_rate_iterm_error(&mut self, measurement: f32) -> f32 {
        let setpoint = self.roll.rate_pid.setpoint();
        // iterm_error is just `setpoint - measurement`, if there is no iterm relax

        if self.use_iterm_relax {
            const ITERM_RELAX_SETPOINT_THRESHOLD: f32 = 40.0;
            let setpoint_threshold_dps =
                if self.use_angle_mode { ITERM_RELAX_SETPOINT_THRESHOLD * 0.2 } else { ITERM_RELAX_SETPOINT_THRESHOLD };
            (setpoint - measurement) * self.roll.calculate_rate_iterm_relax_factor(setpoint, setpoint_threshold_dps)
        } else {
            setpoint - measurement
        }
    }

    #[inline]
    pub fn calculate_pitch_rate_iterm_error(&mut self, measurement: f32) -> f32 {
        let setpoint = self.pitch.rate_pid.setpoint();
        // iterm_error is just `setpoint - measurement`, if there is no iterm relax
        if self.use_iterm_relax {
            const ITERM_RELAX_SETPOINT_THRESHOLD: f32 = 40.0;
            let setpoint_threshold_dps =
                if self.use_angle_mode { ITERM_RELAX_SETPOINT_THRESHOLD * 0.2 } else { ITERM_RELAX_SETPOINT_THRESHOLD };
            (setpoint - measurement) * self.pitch.calculate_rate_iterm_relax_factor(setpoint, setpoint_threshold_dps)
        } else {
            setpoint - measurement
        }
    }
}

#[allow(unused)]
impl FlightController {
    #[inline]
    pub fn roll_rate_ned_dps(gyro_enu_rps: Vector3f32) -> f32 {
        gyro_enu_rps.y.to_degrees()
    }

    #[inline]
    pub fn pitch_rate_ned_dps(gyro_enu_rps: Vector3f32) -> f32 {
        gyro_enu_rps.x.to_degrees()
    }

    #[inline]
    pub fn yaw_rate_ned_dps(gyro_enu_rps: Vector3f32) -> f32 {
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
    pub fn motors_switch_off(&mut self, motor_mixer: &mut MotorMixer) {
        motor_mixer.motors_switch_off();
        //self.ground_mode = true;
        self.switch_pid_integration_off();
    }

    pub fn motors_switch_on(&mut self, motor_mixer: &mut MotorMixer) {
        // don't allow motors to be switched on if the sensor fusion has not initialized
        if !self.vehicle_controller().sensor_fusion_filter_is_initializing() {
            motor_mixer.motors_switch_on();
            // reset the PID integral values when we switch the motors on
            self.switch_pid_integration_on();
        }
    }

    pub fn switch_pid_integration_on(&mut self) {
        self.roll.rate_pid.switch_integration_on();
        self.pitch.rate_pid.switch_integration_on();
        self.yaw_rate_pid.switch_integration_on();
    }

    pub fn switch_pid_integration_off(&mut self) {
        self.roll.rate_pid.switch_integration_off();
        self.pitch.rate_pid.switch_integration_off();
        self.yaw_rate_pid.switch_integration_off();
    }

    pub fn reset_pid_integrals(&mut self) {
        self.roll.rate_pid.reset_integral();
        self.pitch.rate_pid.reset_integral();
        self.yaw_rate_pid.reset_integral();
    }

    /// Set the flight stabilization mode required my the `RcMode`.
    pub fn set_stabilization_mode(&mut self, rc_modes: BitSet64) {
        const ANGLE_MODES: u64 = (1u64 << RcMode::ANGLE)
            | (1u64 << RcMode::ALTITUDE_HOLD)
            | (1u64 << RcMode::POSITION_HOLD)
            | (1u64 << RcMode::FAILSAFE)
            | (1u64 << RcMode::GPS_RESCUE)
            | (1u64 << RcMode::AUTOPILOT);

        let stabilization_mode = if rc_modes.test(RcMode::HORIZON) {
            FlightStabilizationMode::LevelRace
        } else if rc_modes.contains_any(ANGLE_MODES) {
            FlightStabilizationMode::Angle
        } else {
            FlightStabilizationMode::Rate
        };

        if stabilization_mode != self.stabilization_mode {
            self.stabilization_mode = stabilization_mode;
            self.reset_pid_integrals();
        }
    }

    pub fn recover_from_yaw_spin(&mut self, gyro_rps: Vector3f32, delta_t: f32) -> Vector4f32 {
        _ = self;
        _ = gyro_rps;
        _ = delta_t;
        Vector4f32::default()
    }

    pub fn apply_crash_flip_to_motors(&mut self, _gyro_rps: Vector3f32, _delta_t: f32) -> Vector4f32 {
        _ = self;
        Vector4f32::default()
    }

    pub fn update_setpoints(&mut self, controls: RcControls, rc_modes: BitSet64) {
        //detect_crash_or_spin();

        self.set_stabilization_mode(rc_modes);

        // output throttle may be changed by spin recovery
        self.motor_commands_throttle = controls.throttle_stick;

        //
        // Roll axis
        //
        // Pushing the ROLL stick to the right gives a positive value of roll_stick and we want this to be left side up.
        // For NED left side up is positive roll, so sign of setpoint is same sign as roll_stick.
        // So sign of roll_stick is left unchanged.
        if !self.use_angle_mode {
            self.roll.rate_pid.set_setpoint(controls.roll_stick_dps);
        }
        self.roll.angle_pid.set_setpoint(controls.roll_stick_degrees);

        //
        // Pitch axis
        //
        // Pushing the  PITCH stick forward gives a positive value of _pitch_stick and we want this to be nose down.
        // For NED nose down is negative pitch, so sign of setpoint is opposite sign as _pitch_stick.
        // So sign of _pitch_stick is negated.
        if !self.use_angle_mode {
            self.pitch.rate_pid.set_setpoint(-controls.pitch_stick_dps);
        }
        self.pitch.angle_pid.set_setpoint(-controls.pitch_stick_degrees);

        //
        // Yaw axis
        //
        // Pushing the YAW stick to the right gives a positive value of _yaw_stick and we want this to be nose right.
        // For NED nose left is positive yaw, so sign of setpoint is same as sign of _yaw_stick.
        // So sign of _yaw_stick is left unchanged.
        self.yaw_rate_pid.set_setpoint(controls.yaw_stick_dps);

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
        self.use_angle_mode = (self.stabilization_mode >= FlightStabilizationMode::Angle) && !self.ground_mode;

        self.use_level_race_mode = (self.stabilization_mode == FlightStabilizationMode::LevelRace)
            || (self.flight_mode_config.level_race_mode != 0);
    }
}

impl FlightController {
    /// NOTE: CALLED FROM WITHIN THE GYRO/PID TASK.
    ///
    /// In angle mode, the roll and pitch angles are used to set the setpoints for the rollRate and pitchRate PIDs.
    /// Level Race Mode (aka NFE(Not Fast Enough) mode) is equivalent to angle mode on roll and acro mode on pitch.
    #[inline]
    fn update_rate_setpoints_for_angle_mode(&mut self, orientation: Quaternionf32, delta_t: f32) {
        self.angle_mode_calculation_state.update(
            &mut self.roll,
            &mut self.pitch,
            orientation,
            self.stabilization_mode,
            delta_t,
        );
    }
}

/// State machine to calculate setpoints for angle mode.
/// Calculates alternates between roll and pitch axis on each iteration.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum AngleModeCalculationState {
    #[default]
    CalculateRoll,
    CalculatePitch,
}

impl AngleModeCalculationState {
    pub const fn new() -> Self {
        Self::CalculateRoll
    }
}

#[allow(unused)]
impl AngleModeCalculationState {
    /// Perform one step of the state machine.
    /// Alternates between roll and pitch axis on each step.
    fn update(
        &mut self,
        roll: &mut FcTilt,
        pitch: &mut FcTilt,
        orientation: Quaternionf32,
        stabilization_mode: FlightStabilizationMode,
        dt: f32,
    ) {
        *self = match core::mem::take(self) {
            Self::CalculateRoll => {
                let roll_angle_degrees = FlightController::roll_angle_degrees_ned(orientation);
                let roll_angle_delta = (roll_angle_degrees - roll.angle_pid.previous_measurement())
                    .filter_using(&mut roll.angle_dterm_filter);

                // calculate roll rate setpoint in degrees, range is [-roll.max_angle_degrees, roll.max_angle_degrees], typically [-60, 60]
                let roll_rate_setpoint_degrees = roll.angle_pid.update_delta(roll_angle_degrees, roll_angle_delta, dt);

                // convert to value in range [-1.0, 1.0] to be used for the roll rate setpoint
                let roll_rate_setpoint_dps =
                    (roll_rate_setpoint_degrees / roll.max_angle_degrees).clamp(-1.0, 1.0) * roll.max_rate_dps;

                roll.rate_pid.set_setpoint(roll_rate_setpoint_dps);

                if stabilization_mode == FlightStabilizationMode::LevelRace {
                    // in level race mode we use angle mode on roll, acro mode on pitch, so keep state as CalculateRoll
                    Self::CalculateRoll
                } else {
                    Self::CalculatePitch
                }
            }

            Self::CalculatePitch => {
                let pitch_angle_degrees = FlightController::pitch_angle_degrees_ned(orientation);
                let pitch_angle_delta = (pitch_angle_degrees - pitch.angle_pid.previous_measurement())
                    .filter_using(&mut pitch.angle_dterm_filter);

                // calculate pitch rate setpoint in degrees, range is [-pitch.max_angle_degrees, pitch.max_angle_degrees], typically [-60, 60]
                let pitch_rate_setpoint_degrees =
                    pitch.angle_pid.update_delta(pitch_angle_degrees, pitch_angle_delta, dt);

                // convert to value in range [-1.0, 1.0] to be used for the pitch rate setpoint
                let pitch_rate_setpoint_dps =
                    (pitch_rate_setpoint_degrees / pitch.max_angle_degrees).clamp(-1.0, 1.0) * pitch.max_rate_dps;

                pitch.rate_pid.set_setpoint(pitch_rate_setpoint_dps);

                Self::CalculateRoll
            }
        }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<FlightController>();
        is_full::<FlightStabilizationMode>();
        is_full::<FcTilt>();
        is_full::<AngleModeCalculationState>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let flight_controller = FlightController::new();
        assert_eq!(FlightStabilizationMode::Rate, flight_controller.stabilization_mode);
    }
}
