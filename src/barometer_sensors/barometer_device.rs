use super::BarometerI2cError;
use embassy_time::{Duration, Instant};

/// The common interface for barometer.
#[allow(unused)]
pub trait BarometerDevice {
    /// Initializes the sensor and returns the actual configured output data rate (ODR) in Hz.
    async fn init(&mut self) -> Result<Duration, BarometerI2cError>;

    async fn make_reading(&mut self) -> Result<(Instant, f32, f32), BarometerI2cError>;
}
