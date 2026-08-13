use crate::barometer_sensors::{BarometerMessage, barometer::RxBarometer};

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

impl RxBarometer for BarometerMock {
    fn message(&self) -> BarometerMessage {
        BarometerMessage::default()
    }
}
