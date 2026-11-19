use embassy_time::{Duration, Instant};

use super::BarometerI2cError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarometerMock {}

impl Default for BarometerMock {
    fn default() -> Self {
        Self::new()
    }
}

impl BarometerMock {
    pub const fn new() -> Self {
        Self {}
    }
}

impl BarometerMock {
    pub async fn init(&self) -> Result<Duration, BarometerI2cError> {
        core::future::ready(()).await;

        Ok(Duration::MIN)
    }

    pub async fn make_reading(&mut self) -> Result<(Instant, f32, f32), BarometerI2cError> {
        core::future::ready(()).await;
        _ = self;
        Ok((Instant::MIN, 0.0, 0.0))
    }
}
