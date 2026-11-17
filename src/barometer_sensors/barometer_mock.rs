use embassy_time::Duration;

use super::{BarometerI2cError, BarometerMessage};

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

//impl BarometerDevice for BarometerMock {
impl BarometerMock {
    pub async fn init(&self) -> Result<Duration, BarometerI2cError> {
        core::future::ready(()).await;

        Ok(Duration::MIN)
    }

    pub async fn make_reading(&mut self) -> Result<(), BarometerI2cError> {
        core::future::ready(()).await;
        _ = self;
        Ok(())
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn message(&self) -> BarometerMessage {
        _ = self;
        BarometerMessage::default()
    }
}
