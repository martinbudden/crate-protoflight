use crate::{flight::FlightController, flight::flight_controller_config::PidConfig};
use pidsk_controller::PidskGainsf32;

impl FlightController {
    // Betaflight compatible scale factors.
    const PID_SCALE_FACTORS: PidskGainsf32 = PidskGainsf32 {
        kp: 0.032_029,
        ki: 0.244_381,
        kd: 0.000_529,
        ks: 0.01, // TODO: provisional value
        kk: 0.013_754,
    };

    pub fn calculate_gains(pid_config: PidConfig) -> PidskGainsf32 {
        PidskGainsf32 {
            kp: f32::from(pid_config.kp) * Self::PID_SCALE_FACTORS.kp,
            ki: f32::from(pid_config.ki) * Self::PID_SCALE_FACTORS.ki,
            kd: f32::from(pid_config.kd) * Self::PID_SCALE_FACTORS.kd,
            ks: f32::from(pid_config.ks) * Self::PID_SCALE_FACTORS.ks,
            kk: f32::from(pid_config.kk) * Self::PID_SCALE_FACTORS.kk,
        }
    }

    /*/// Set the PID gains for the PID with the given index.
    /// Integration is switched off, so that there is no integral windup.
    pub fn set_pid_gains(&mut self, index: usize, pid_config: PidConfig) {
        let gains = PidskGainsf32 {
            kp: f32::from(pid_config.kp) * Self::PID_SCALE_FACTORS.kp,
            ki: f32::from(pid_config.ki) * Self::PID_SCALE_FACTORS.ki,
            kd: f32::from(pid_config.kd) * Self::PID_SCALE_FACTORS.kd,
            ks: f32::from(pid_config.ks) * Self::PID_SCALE_FACTORS.ks,
            kk: f32::from(pid_config.kk) * Self::PID_SCALE_FACTORS.kk,
        };
        // update pid_gains copy
        self.pid_gains[index] = gains;

        // update the corresponding PID controller
        match index {
            Self::ROLL_RATE_DPS => {
                self.pid_roll_rate.set_gains(gains);
                self.pid_roll_rate.switch_integration_off();
                self.pid_roll_rate.set_setpoint(0.0);
            }
            Self::PITCH_RATE_DPS => {
                self.pid_pitch_rate.set_gains(gains);
                self.pid_pitch_rate.switch_integration_off();
                self.pid_pitch_rate.set_setpoint(0.0);
            }
            Self::YAW_RATE_DPS => {
                self.pid_yaw_rate.set_gains(gains);
                self.pid_yaw_rate.switch_integration_off();
                self.pid_yaw_rate.set_setpoint(0.0);
            }
            Self::ROLL_ANGLE_DEGREES => {
                self.pid_roll_angle.set_gains(gains);
                self.pid_roll_angle.switch_integration_off();
                self.pid_roll_angle.set_setpoint(0.0);
            }
            Self::PITCH_ANGLE_DEGREES => {
                self.pid_pitch_angle.set_gains(gains);
                self.pid_pitch_angle.switch_integration_off();
                self.pid_pitch_angle.set_setpoint(0.0);
            }
            _ => {}
        }
    }*/

    /*/// Return the PID gains config for the given index.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    #[allow(unused)]
    pub fn pid_config(&self, index: usize) -> PidConfig {
        PidConfig {
            kp: (self.pid_gains[index].kp / Self::PID_SCALE_FACTORS.kp) as u8,
            ki: (self.pid_gains[index].ki / Self::PID_SCALE_FACTORS.ki) as u8,
            kd: (self.pid_gains[index].kd / Self::PID_SCALE_FACTORS.kd) as u8,
            ks: (self.pid_gains[index].ks / Self::PID_SCALE_FACTORS.ks) as u8,
            kk: (self.pid_gains[index].kk / Self::PID_SCALE_FACTORS.kk) as u8,
        }
    }*/
}
