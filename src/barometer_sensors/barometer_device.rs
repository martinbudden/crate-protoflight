use num_traits::Float;

use crate::barometer_sensors::{barometer::BarometerError, i2c::I2cError};
/// The common interface for barometer.
pub trait BarometerDevice {
    /// Returns sample rate or error.
    //pub async fn init(&self) -> Result<u32, BarometerError<I2cError>> {
    async fn init(&mut self) -> Result<u32, BarometerError<I2cError>>;

    async fn make_reading(&mut self);
    fn message(&self) -> BarometerMessage;
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(derive_more::Display))]
#[cfg_attr(feature = "std", display("Baro{{a:{altitude_m}, p:{pressure_pascals}, t:{temperature_celsius}}}"))]
pub struct BarometerMessage {
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
        Self { altitude_m: 0.0, altitude_m_i32: 0, pressure_pascals: 0.0, temperature_celsius: 0.0 }
    }
}

impl BarometerMessage {
    pub fn calculate_altitude_meters(pressure: f32, pressure_at_reference_altitude: f32) -> f32 {
        44330.0 * (1.0 - (pressure / pressure_at_reference_altitude).powf(0.1903))
    }
}
