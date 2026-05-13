use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use static_cell::StaticCell;

use motor_mixers::{MotorMixerMessage, MotorMixerOutput, MotorMixerQuadXPwm};

pub(crate) static MOTOR_MIXER_CTX: StaticCell<MotorMixerContext> = StaticCell::new();
// --- MOTOR_SIGNAL ---
// High-speed trigger for Motors (8kHz)
// no watch count, since a signal can only have one watcher.
pub static MOTOR_MIXER_SIGNAL: Signal<CriticalSectionRawMutex, MotorMixerMessage> = Signal::new();

/// Context for motor_mixer_task.
pub struct MotorMixerContext {
    pub motor_mixer: MotorMixerQuadXPwm,
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
