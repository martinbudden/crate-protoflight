#[cfg(feature = "serde")]
use {
    sequential_storage::map::PostcardValue,
    serde::{Deserialize, Serialize},
};

#[cfg(feature = "rpm_filters")]
use motor_mixers::{RpmNotchFilterBank, RpmNotchFilterBankConfig, RpmNotchFilters};
use signal_filters::{BiquadFilterVector3df32, MedianFilter3f32, Pt1FilterVector3df32, SignalFilter};
use vqm::Vector3df32;

/// Configuration data for the IMU filters bank.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ImuFilterBankConfig {
    pub acc_lpf_hz: u16,
    pub gyro_lpf1_hz: u16,
    pub gyro_lpf2_hz: u16,
    pub gyro_notch1_hz: u16,
    pub gyro_notch1_cutoff: u16,
    pub gyro_notch2_hz: u16,
    pub gyro_notch2_cutoff: u16,
    #[cfg(feature = "rpm_filters")]
    pub rpm_filters: RpmNotchFilterBankConfig,
}

#[cfg(feature = "serde")]
impl PostcardValue<'_> for ImuFilterBankConfig {}

impl ImuFilterBankConfig {
    pub const fn new() -> Self {
        Self {
            acc_lpf_hz: 100,
            gyro_lpf1_hz: 0,   // switched off
            gyro_lpf2_hz: 250, // this is an anti-alias filter and shouldn't be disabled
            gyro_notch1_hz: 0,
            gyro_notch1_cutoff: 0,
            gyro_notch2_hz: 0,
            gyro_notch2_cutoff: 0,
            #[cfg(feature = "rpm_filters")]
            rpm_filters: RpmNotchFilterBankConfig::new(),
        }
    }
}

impl Default for ImuFilterBankConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Bank of filters to filter the IMU values before they are sent to sensor fusion and the flight controller.
/// Includes low-pass, skew, notch, and RMP filters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImuFilterBank {
    motor_count: usize,
    config: ImuFilterBankConfig,
    acc_lpf: Pt1FilterVector3df32,
    gyro_skew: [MedianFilter3f32; 3],
    gyro_lpf1: Pt1FilterVector3df32,
    gyro_lpf2: Pt1FilterVector3df32,
    gyro_notch1: BiquadFilterVector3df32,
    gyro_notch2: BiquadFilterVector3df32,
    #[cfg(feature = "rpm_filters")]
    rpm_filters: RpmNotchFilterBank,
}

impl Default for ImuFilterBank {
    fn default() -> Self {
        Self::new()
    }
}

impl ImuFilterBank {
    pub const fn with_config(config: ImuFilterBankConfig) -> Self {
        Self {
            motor_count: 4,
            config,
            acc_lpf: Pt1FilterVector3df32::new(),
            gyro_skew: [MedianFilter3f32::new(), MedianFilter3f32::new(), MedianFilter3f32::new()],
            gyro_lpf1: Pt1FilterVector3df32::new(),
            gyro_lpf2: Pt1FilterVector3df32::new(),
            gyro_notch1: BiquadFilterVector3df32::new(),
            gyro_notch2: BiquadFilterVector3df32::new(),
            #[cfg(feature = "rpm_filters")]
            rpm_filters: RpmNotchFilterBank::new(),
        }
    }

    pub const fn new() -> Self {
        Self::with_config(ImuFilterBankConfig::new())
    }
}

impl ImuFilterBank {
    #[allow(unused)]
    pub fn set_config(&mut self, config: ImuFilterBankConfig, delta_t: f32) {
        self.config = config;
        self.acc_lpf.set_cutoff_frequency_and_reset(f32::from(config.acc_lpf_hz), delta_t);
        self.gyro_lpf1.set_cutoff_frequency_and_reset(f32::from(config.gyro_lpf1_hz), delta_t);
        self.gyro_lpf2.set_cutoff_frequency_and_reset(f32::from(config.gyro_lpf1_hz), delta_t);
        self.gyro_notch1.set_notch_frequency(f32::from(config.gyro_notch1_hz), f32::from(config.gyro_notch1_cutoff));
        self.gyro_notch2.set_notch_frequency(f32::from(config.gyro_notch2_hz), f32::from(config.gyro_notch2_cutoff));
    }
}

/// Trait to allow IMU to be filtered by `ImuFilterBank`.
pub trait FilterAccGyro {
    fn state(&self) -> &ImuFilterBank;
    fn state_mut(&mut self) -> &mut ImuFilterBank;
    fn config(&self) -> &ImuFilterBankConfig;

    fn update(&mut self, acc: Vector3df32, gyro_rps: Vector3df32, delta_t: f32) -> (Vector3df32, Vector3df32);
}

impl FilterAccGyro for ImuFilterBank {
    fn state(&self) -> &ImuFilterBank {
        self
    }
    fn state_mut(&mut self) -> &mut ImuFilterBank {
        self
    }
    fn config(&self) -> &ImuFilterBankConfig {
        &self.state().config
    }

    fn update(&mut self, mut acc: Vector3df32, mut gyro_rps: Vector3df32, _delta_t: f32) -> (Vector3df32, Vector3df32) {
        if self.config().acc_lpf_hz != 0 {
            acc = self.state_mut().acc_lpf.update(acc);
        }

        gyro_rps.x = self.state_mut().gyro_skew[0].update(gyro_rps.x);
        gyro_rps.y = self.state_mut().gyro_skew[1].update(gyro_rps.y);
        gyro_rps.z = self.state_mut().gyro_skew[2].update(gyro_rps.z);

        if self.config().gyro_lpf1_hz != 0 {
            gyro_rps = self.state_mut().gyro_lpf1.update(gyro_rps);
        }
        if self.config().gyro_lpf2_hz != 0 {
            gyro_rps = self.state_mut().gyro_lpf2.update(gyro_rps);
        }
        if self.config().gyro_notch1_hz != 0 {
            gyro_rps = self.state_mut().gyro_notch1.update(gyro_rps);
        }
        if self.config().gyro_notch2_hz != 0 {
            gyro_rps = self.state_mut().gyro_notch2.update(gyro_rps);
        }
        #[cfg(feature = "rpm_filters")]
        for ii in 0..self.state().motor_count {
            gyro_rps = self.state_mut().rpm_filters.update(gyro_rps, ii);
        }

        (acc, gyro_rps)
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
        is_full::<ImuFilterBankConfig>();
    #[cfg(feature = "serde")]
        is_config::<ImuFilterBankConfig>();
        is_full::<ImuFilterBank>();
    }
    #[test]
    fn test_new() {
        let config = ImuFilterBankConfig::new();
        assert_eq!(100, config.acc_lpf_hz);
    }
}
