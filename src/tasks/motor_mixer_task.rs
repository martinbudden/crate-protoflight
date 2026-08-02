use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use motor_mixers::{
    MixerConfig, MotorConfig, MotorMixerCommon, MotorMixerMessage, MotorMixerOutput, MotorMixerQuadXPwm,
};

// --- MOTOR_SIGNAL ---
// High-speed trigger for Motors (8kHz)
// no watch count, since a signal can only have one watcher.
pub static MOTOR_MIXER_SIGNAL: Signal<CriticalSectionRawMutex, MotorMixerMessage> = Signal::new();

/// Context for `motor_mixer_task`.
pub struct MotorMixerContext {
    pub motor_mixer: MotorMixerQuadXPwm,
}

impl MotorMixerContext {
    pub const fn new(mixer_config: MixerConfig, motor_config: MotorConfig) -> Self {
        Self { motor_mixer: MotorMixerQuadXPwm::new(MotorMixerCommon::with_config(mixer_config, motor_config)) }
    }
}

#[embassy_executor::task]
pub async fn motor_mixer_task(ctx: &'static mut MotorMixerContext) {
    loop {
        // wait for the motor mixer message from the gyro_pid task
        let motor_mixer_message = MOTOR_MIXER_SIGNAL.wait().await;
        // and use it to output to the motors.
        ctx.motor_mixer.output_to_motors(motor_mixer_message);
    }
}
