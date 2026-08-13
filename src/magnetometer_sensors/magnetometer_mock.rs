use crate::magnetometer_sensors::{MagnetometerMessage, magnetometer::RxMagnetometer};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MagnetometerMock {}

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

impl RxMagnetometer for MagnetometerMock {
    fn message(&self) -> MagnetometerMessage {
        MagnetometerMessage::default()
    }
}
