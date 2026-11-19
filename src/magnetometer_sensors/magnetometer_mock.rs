use embassy_time::{Duration, Instant};

use super::MagnetometerI2cError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MagnetometerMock;

impl Default for MagnetometerMock {
    fn default() -> Self {
        Self::new()
    }
}

impl MagnetometerMock {
    pub const fn new() -> Self {
        Self {}
    }
}

impl MagnetometerMock {
    pub async fn init(&self) -> Result<Duration, MagnetometerI2cError> {
        core::future::ready(()).await;

        Ok(Duration::MIN)
    }

    pub async fn make_reading(&mut self) -> Result<(Instant, [i16; 3]), MagnetometerI2cError> {
        core::future::ready(()).await;
        Ok((Instant::now(), [0; 3]))
    }
}
#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<MagnetometerMock>();
    }
}
