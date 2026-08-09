use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use motor_mixers::{MixerConfig, MotorConfig, MotorDriver, MotorMixer, MotorMixerMessage};
#[cfg(feature = "rpm_filters")]
use motor_mixers::{RpmNotchFilterBank, RpmNotchFilterBankConfig};

// --- MOTOR_SIGNAL ---
// High-speed trigger for Motors (8kHz)
// no watch count, since a signal can only have one watcher.
pub static MOTOR_MIXER_SIGNAL: Signal<CriticalSectionRawMutex, MotorMixerMessage> = Signal::new();

/// Context for `motor_mixer_task`.
#[allow(unused)]
#[rustfmt::skip]
pub struct MotorMixerContext {
    pub motor_mixer: MotorMixer,
    #[cfg(feature = "rpm_filters")] rpm_notch_filters: RpmNotchFilterBank,
    #[cfg(feature = "rpm_filters")] rpm_filter_iteration_count: usize,
}

impl MotorMixerContext {
    #[cfg(feature = "rpm_filters")]
    pub fn new(
        mixer_config: MixerConfig,
        motor_config: MotorConfig,
        motor_driver: MotorDriver,
        rpm_notch_filter_config: RpmNotchFilterBankConfig,
        looptime_seconds: f32,
    ) -> Self {
        let rpm_notch_filters = RpmNotchFilterBank::new(rpm_notch_filter_config, looptime_seconds);
        // rpm_filter_harmonics_count calculated in RpmNotchFilterBank::new()
        // We need to complete the rpm_filter iterations before the next time rpm_filter.start() is called.
        // So, for example, if there are 2 harmonics and 4 motors that gives 8 iterations in total.
        // So if output_denominator is 2, then we need to do 4 iterations.
        // If output denominator is 3, then we need to do 3 iterations.
        let rpm_filter_iteration_count = 8;
        //(rpm_notch_filters.rpm_filter_harmonics_count() * Self::MOTOR_COUNT).div_ceil(common.output_denominator());
        Self {
            motor_mixer: MotorMixer::new(mixer_config, motor_config, motor_driver),
            rpm_notch_filters,
            rpm_filter_iteration_count,
        }
    }
    #[cfg(not(feature = "rpm_filters"))]
    pub fn new(mixer_config: MixerConfig, motor_config: MotorConfig, motor_driver: MotorDriver) -> Self {
        Self { motor_mixer: MotorMixer::new(mixer_config, motor_config, motor_driver) }
    }
}

#[embassy_executor::task]
pub async fn motor_mixer_task(ctx: &'static mut MotorMixerContext) {
    loop {
        // wait for the motor mixer message from the gyro_pid task
        let motor_mixer_message = MOTOR_MIXER_SIGNAL.wait().await;
        // and use it to output to the motors.
        ctx.motor_mixer.output_to_motors(motor_mixer_message);

        #[cfg(feature = "rpm_filters")]
        {
            if let Some(frequencies) = ctx.motor_mixer.motor_frequencies() {
                // Start the notch filter state machine.
                ctx.rpm_notch_filters.start_updating_filter_frequencies(frequencies);
            }

            for _ in 0..ctx.rpm_filter_iteration_count {
                // Run one iteration of the state machine.
                ctx.rpm_notch_filters.update_filter_frequencies_step();
            }
        }
    }
}
