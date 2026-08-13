#![cfg(feature = "rangefinder")]

use crate::rangefinder_sensors::{RangefinderMessage, rangefinder::RangefinderDevice};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangefinderHcsr04 {}

impl Default for RangefinderHcsr04 {
    fn default() -> Self {
        Self::new()
    }
}

impl RangefinderHcsr04 {
    pub const fn new() -> Self {
        Self {}
    }
}

impl RangefinderDevice for RangefinderHcsr04 {
    fn message(&self) -> RangefinderMessage {
        RangefinderMessage::default()
    }
}
