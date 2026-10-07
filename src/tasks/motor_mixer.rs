use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use motor_mixers::{MixerConfig, MotorConfig, MotorDriver, MotorMixer, MotorMixerMessage};
#[cfg(feature = "rpm_filters")]
use motor_mixers::{RpmNotchFilterBank, RpmNotchFilterBankConfig};
use static_cell::StaticCell;

// --- MOTOR_SIGNAL ---
// High-speed trigger for Motors (8kHz)
// no watch count, since a signal can only have one watcher.
pub static MOTOR_MIXER_SIGNAL: Signal<CriticalSectionRawMutex, MotorMixerMessage> = Signal::new();

static MOTOR_MIXER_CTX: StaticCell<MotorMixerContext> = StaticCell::new();

/// Context for `motor_mixer` task.
#[allow(unused)]
#[rustfmt::skip]
pub struct MotorMixerContext {
    pub motor_mixer: MotorMixer,
    #[cfg(feature = "rpm_filters")] rpm_notch_filters: RpmNotchFilterBank,
    #[cfg(feature = "rpm_filters")] rpm_filter_iteration_count: usize,
}

pub fn init(
    mixer_config: MixerConfig,
    motor_config: MotorConfig,
    motor_driver: MotorDriver,
    #[cfg(feature = "rpm_filters")] rpm_notch_filter_config: RpmNotchFilterBankConfig,
    #[cfg(feature = "rpm_filters")] looptime_seconds: f32,
) -> &'static mut MotorMixerContext {
    // rpm_filter_harmonics_count calculated in RpmNotchFilterBank::new()
    // We need to complete the rpm_filter iterations before the next time rpm_filter.start() is called.
    // So, for example, if there are 2 harmonics and 4 motors that gives 8 iterations in total.
    // So if output_denominator is 2, then we need to do 4 iterations.
    // If output denominator is 3, then we need to do 3 iterations.
    //(rpm_notch_filters.rpm_filter_harmonics_count() * Self::MOTOR_COUNT).div_ceil(common.output_denominator());
    #[cfg(feature = "rpm_filters")]
    let ctx = MotorMixerContext {
        motor_mixer: MotorMixer::new(mixer_config, motor_config, motor_driver),
        rpm_notch_filters: RpmNotchFilterBank::new(rpm_notch_filter_config, looptime_seconds),
        rpm_filter_iteration_count: 8,
    };
    #[cfg(not(feature = "rpm_filters"))]
    let ctx = MotorMixerContext { motor_mixer: MotorMixer::new(mixer_config, motor_config, motor_driver) };
    MOTOR_MIXER_CTX.init(ctx)
}

#[embassy_executor::task]
pub async fn run(ctx: &'static mut MotorMixerContext) {
    loop {
        // wait for the motor mixer message from the gyro_pid task
        let motor_mixer_message = MOTOR_MIXER_SIGNAL.wait().await;
        // and use it to output to the motors.
        ctx.motor_mixer.output_to_motors(motor_mixer_message).await;

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
