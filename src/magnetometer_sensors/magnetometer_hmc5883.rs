#![cfg(feature = "magnetometer")]

use crate::magnetometer_sensors::{MagnetometerMessage, magnetometer::RxMagnetometer};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MagnetometerHmc5883 {}

impl Default for MagnetometerHmc5883 {
    fn default() -> Self {
        Self::new()
    }
}

impl MagnetometerHmc5883 {
    pub const fn new() -> Self {
        Self {}
    }
}

impl RxMagnetometer for MagnetometerHmc5883 {
    fn message(&self) -> MagnetometerMessage {
        MagnetometerMessage::default()
    }
}
