use crate::barometer_sensors::{BarometerDevice, BarometerMessage, barometer::BarometerError, i2c::I2cError};

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
    pub async fn init(&self) -> Result<u32, BarometerError<I2cError>> {
        //async fn init(&mut self) -> Result<u32, ()> {
        // Placeholder: explicitly await an immediately ready inline future
        core::future::poll_fn(|_| core::task::Poll::Ready(())).await;

        Ok(40)
    }

    pub async fn make_reading(&mut self) {
        // Placeholder: explicitly await an immediately ready inline future
        core::future::poll_fn(|_| core::task::Poll::Ready(())).await;
        _ = self;
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn message(&self) -> BarometerMessage {
        _ = self;
        BarometerMessage::default()
    }
}
