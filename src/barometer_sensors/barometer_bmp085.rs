#![cfg(feature = "barometer")]

use crate::barometer_sensors::{BarometerMessage, barometer::BarometerDevice};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarometerBmp085 {}

impl Default for BarometerBmp085 {
    fn default() -> Self {
        Self::new()
    }
}

impl BarometerBmp085 {
    pub const fn new() -> Self {
        Self {}
    }
}

impl BarometerDevice for BarometerBmp085 {
    fn message(&self) -> BarometerMessage {
        BarometerMessage::default()
    }
}
