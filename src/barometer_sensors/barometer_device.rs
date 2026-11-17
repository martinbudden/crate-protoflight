use embassy_time::{Duration, Instant};

use crate::i2c_bus::I2cError;

/// The common interface for barometer.
#[allow(unused)]
pub trait BarometerDevice {
    /// Initializes the sensor and returns the actual configured output data rate (ODR) in Hz.
    async fn init(&mut self) -> Result<Duration, BarometerI2cError>;

    async fn make_reading(&mut self) -> Result<(), BarometerI2cError>;
    fn message(&self) -> BarometerMessage;
}

#[allow(unused)]
#[derive(Debug)]
pub enum BarometerError<E> {
    I2c(E),
    InvalidChipId { expected: u8, actual: u8 },
    InvalidCalibration,
    InvalidData,
}

pub type BarometerI2cError = BarometerError<I2cError>;

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "host", derive(derive_more::Display))]
#[cfg_attr(feature = "host", display("Baro{{a:{altitude_m}, p:{pressure_pascals}, t:{temperature_celsius}}}"))]
pub struct BarometerMessage {
    pub timestamp: Instant,
    pub altitude_m: f32,
    pub altitude_m_i32: i32,
    pub pressure_pascals: f32,
    pub temperature_celsius: f32,
}

impl Default for BarometerMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl BarometerMessage {
    pub const fn new() -> Self {
        Self { timestamp: Instant::MIN, altitude_m: 0.0, altitude_m_i32: 0, pressure_pascals: 0.0, temperature_celsius: 0.0 }
    }
}

impl BarometerMessage {
    #[allow(unused)]
    pub fn calculate_altitude_meters(pressure: f32, pressure_at_reference_altitude: f32) -> f32 {
        44330.0 * (1.0 - (pressure / pressure_at_reference_altitude).powf(0.1903))
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<BarometerMessage>();
    }
}
