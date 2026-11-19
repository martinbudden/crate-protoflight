use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Instant};

use crate::boards::I2cDeviceBlocking;

pub type SharedI2cBus = Mutex<NoopRawMutex, I2cDeviceBlocking>;
pub type I2cError = <I2cDeviceBlocking as embedded_hal::i2c::ErrorType>::Error;

#[allow(unused)]
#[derive(Debug)]
pub enum BarometerError<E> {
    I2c(E),
    InvalidChipId { expected: u8, actual: u8 },
    InvalidCalibration,
    InvalidData,
    TemperatureOversamplingNotSet,
    PressureOversamplingNotSet,
    InSleepMode,
    IdleTimeout,
}

pub type BarometerI2cError = BarometerError<I2cError>;

/// The common interface for barometer.
#[allow(unused)]
pub trait BarometerDevice {
    /// Initializes the sensor and returns the actual configured output data rate (ODR) in Hz.
    async fn init(&mut self) -> Result<Duration, BarometerI2cError>;

    async fn make_reading(&mut self) -> Result<(Instant, f32, f32), BarometerI2cError>;
}

#[cfg(test)]
mod test_traits {
    use super::*;
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<BarometerI2cError>();
    }
}
