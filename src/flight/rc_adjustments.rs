#![allow(unused)]
use radio_controllers::{
    RatesConfig, RcAdjustmentConfig, RcAdjustmentMode, RcAdjustmentRange, RcContinuosAdjustmentState,
    RcTimedAdjustmentState,
};
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

use crate::{
    config::{ConfigItem, ConfigPublisher, FastConfigPublisher, GLOBAL_CONFIG},
    flight::FlightController,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
enum RcAdjustment {
    #[default]
    None,

    // Rates
    ThrottleExpo,
    RateProfile,
    RcRate,
    RcExpo,
    RollRate,
    RollRcRate,
    RollRcExpo,
    PitchRate,
    PitchRcRate,
    PitchRcExpo,
    PitchRollRate,
    YawRate,
    YawRcRate,

    // PIDs
    RollP,
    RollI,
    RollD,
    RollK,
    PitchP,
    PitchI,
    PitchD,
    PitchK, // called PITCH_F in betaflight
    PitchRollP,
    PitchRollI,
    PitchRollD,
    PitchRollK,
    YawP,
    YawI,
    YawD,
    YawK,

    FeedforwardTransition,
    HorizonStrength,
    PidAudio,
    OsdProfile,
    LedProfile,
    LedDimmer,
    FunctionCount,
}

impl RcAdjustment {
    //pub const COUNT: usize = Self::FunctionCount as usize;
    pub const COUNT: usize = 2;
    pub const fn new() -> Self {
        Self::None
    }
}

pub const RC_ADJUSTMENT_CONFIGS: [RcAdjustmentConfig; RcAdjustment::COUNT] = [
    RcAdjustmentConfig {
        adjustment: RcAdjustment::RcRate as u8,
        adjustment_mode: RcAdjustmentMode::Step as u8,
        data: 1,
    },
    RcAdjustmentConfig {
        adjustment: RcAdjustment::RcExpo as u8,
        adjustment_mode: RcAdjustmentMode::Step as u8,
        data: 1,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RcAdjustments {
    pub stepwise_adjustments: [RcTimedAdjustmentState; Self::MAX_RANGE_COUNT],
    pub continuos_adjustments: [RcContinuosAdjustmentState; Self::MAX_RANGE_COUNT],
    pub adjustment_ranges: [RcAdjustmentRange; Self::MAX_RANGE_COUNT],
    pub adjustment_configs: [RcAdjustmentRange; Self::MAX_RANGE_COUNT],
}

impl RcAdjustments {
    const MAX_RANGE_COUNT: usize = 30;
}

impl RcAdjustments {
    pub const fn new() -> Self {
        Self {
            stepwise_adjustments: [RcTimedAdjustmentState::new(); Self::MAX_RANGE_COUNT],
            continuos_adjustments: [RcContinuosAdjustmentState::new(); Self::MAX_RANGE_COUNT],
            adjustment_ranges: [RcAdjustmentRange::new(); Self::MAX_RANGE_COUNT],
            adjustment_configs: [RcAdjustmentRange::new(); Self::MAX_RANGE_COUNT],
        }
    }
}

impl PostcardValue<'_> for RcAdjustments {}

impl Default for RcAdjustments {
    fn default() -> Self {
        Self::new()
    }
}

impl RcAdjustments {
    pub async fn process_adjustments(
        &mut self,
        config_publisher: &ConfigPublisher<'_>,
        fast_config_publisher: &FastConfigPublisher<'_>,
    ) {
        self.process_stepwise_adjustments(config_publisher, fast_config_publisher).await;
        self.process_continuos_adjustments(config_publisher, fast_config_publisher).await;
    }
    async fn process_stepwise_adjustments(
        &mut self,
        config_publisher: &ConfigPublisher<'_>,
        fast_config_publisher: &FastConfigPublisher<'_>,
    ) {
        let mut global_config = GLOBAL_CONFIG.lock().await; // for now
    }
    async fn process_continuos_adjustments(
        &mut self,
        config_publisher: &ConfigPublisher<'_>,
        fast_config_publisher: &FastConfigPublisher<'_>,
    ) {
        let mut global_config = GLOBAL_CONFIG.lock().await; // for now
    }
    fn apply_absolute_rate_adjustment(adjustment: RcAdjustment, rate: u8, rates: &mut RatesConfig) {
        #[allow(clippy::match_same_arms)]
        match adjustment {
            RcAdjustment::ThrottleExpo => {}
            RcAdjustment::RateProfile => {}
            RcAdjustment::RcRate => {}
            RcAdjustment::RcExpo => {}
            RcAdjustment::RollRate => {
                rates.rc_rates[FlightController::FD_ROLL] = rate;
            }
            RcAdjustment::RollRcRate => {}
            RcAdjustment::RollRcExpo => {}
            RcAdjustment::PitchRate => {
                rates.rc_rates[FlightController::FD_PITCH] = rate;
            }
            RcAdjustment::PitchRcRate => {}
            RcAdjustment::PitchRcExpo => {}
            RcAdjustment::PitchRollRate => {
                rates.rc_rates[FlightController::FD_ROLL] = rate;
                rates.rc_rates[FlightController::FD_PITCH] = rate;
            }
            RcAdjustment::YawRate => {}
            RcAdjustment::YawRcRate => {}
            _ => {}
        }
    }

    async fn apply_absolute_adjustment(
        &mut self,
        config_publisher: &ConfigPublisher<'_>,
        fast_config_publisher: &FastConfigPublisher<'_>,
        adjustment: RcAdjustment,
        value: i32,
    ) {
        match adjustment {
            RcAdjustment::ThrottleExpo
            | RcAdjustment::RateProfile
            | RcAdjustment::RcRate
            | RcAdjustment::RcExpo
            | RcAdjustment::RollRate
            | RcAdjustment::RollRcRate
            | RcAdjustment::RollRcExpo
            | RcAdjustment::PitchRate
            | RcAdjustment::PitchRcRate
            | RcAdjustment::PitchRcExpo
            | RcAdjustment::PitchRollRate
            | RcAdjustment::YawRate
            | RcAdjustment::YawRcRate => {
                let mut global_config = GLOBAL_CONFIG.lock().await;

                let mut rates = global_config.rates;
                let old_rates = rates;

                #[allow(clippy::cast_possible_truncation)]
                let new_rate = value.clamp(1, i32::from(RatesConfig::RC_RATES_MAX)).cast_unsigned() as u8;
                Self::apply_absolute_rate_adjustment(adjustment, new_rate, &mut rates);

                //blackbox_log_inflight_adjustment_event(blackbox, ADJUSTMENT_ROLL_RC_RATE, newValue);
                if rates != old_rates {
                    global_config.rates = rates;
                    config_publisher.publish_immediate(ConfigItem::Rates(rates));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<RcAdjustments>();
        is_config::<RcAdjustments>();
    }
    #[test]
    fn test_new() {
        let _ = RcAdjustments::new();
    }
}
