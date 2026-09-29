#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

/// Configuration of failsafe behavior.<br><br>
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct FailsafeConfig {
    pub throttle_pwm: u16,
    pub throttle_low_delay_deciseconds: u16,
    pub recovery_delay_deciseconds: u16, // time of valid rx data needed to allow recovery from failsafe and re-arming
    pub delay_deciseconds: u8,
    pub landing_time_seconds: u8, // time allowed in landing phase before disarm
    pub procedure: FailsafeProcedure,
    pub switch_mode: FailsafeSwitchMode,
    pub stick_threshold_percent: u8, // _stick deflection percentage to exit GPS Rescue procedure
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for FailsafeConfig {}

impl Default for FailsafeConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl FailsafeConfig {
    /// Constructor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            throttle_pwm: 1000, // throttle off
            throttle_low_delay_deciseconds: 100,
            recovery_delay_deciseconds: 5,
            delay_deciseconds: 15,
            landing_time_seconds: 60,
            procedure: FailsafeProcedure::DropIt,
            switch_mode: FailsafeSwitchMode::Stage1,
            stick_threshold_percent: 30,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub enum FailsafeProcedure {
    #[default]
    DropIt = 0,
    AutoLanding = 1,
    GpsRescue = 2,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for FailsafeProcedure {}

impl_try_from_u8!(FailsafeProcedure);

impl FailsafeProcedure {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::DropIt,
            1 => Self::AutoLanding,
            2 => Self::GpsRescue,
            _ => Self::default(),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub enum FailsafeSwitchMode {
    #[default]
    Stage1 = 0,
    Stage2 = 1,
    Kill = 2,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for FailsafeSwitchMode {}

impl_try_from_u8!(FailsafeSwitchMode);

impl FailsafeSwitchMode {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Stage1,
            1 => Self::Stage2,
            2 => Self::Kill,
            _ => Self::default(),
        }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_full_eq<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + Eq + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full_eq::<FailsafeSwitchMode>();
        is_full_eq::<FailsafeProcedure>();
        is_full::<FailsafeConfig>();
        #[cfg(feature = "serde")]
        is_serde::<FailsafeConfig>();
        #[cfg(feature = "storage")]
        is_storage::<FailsafeConfig>();
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    use super::*;

    #[test]
    fn test_new() {
        let failsafe = FailsafeConfig::new();

        assert_eq!(failsafe.throttle_pwm, 1000); // throttle off
        assert_eq!(failsafe.throttle_low_delay_deciseconds, 100);
        assert_eq!(failsafe.recovery_delay_deciseconds, 5);
        assert_eq!(failsafe.delay_deciseconds, 15);
        assert_eq!(failsafe.landing_time_seconds, 60);
        assert_eq!(failsafe.procedure, FailsafeProcedure::DropIt);
        assert_eq!(failsafe.switch_mode, FailsafeSwitchMode::Stage1);
        assert_eq!(failsafe.stick_threshold_percent, 30);
    }
}
