use crate::barometer_sensors::{BarometerMessage, barometer::BarometerDevice};

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

impl BarometerDevice for BarometerMock {
    fn message(&self) -> BarometerMessage {
        BarometerMessage::default()
    }
}
