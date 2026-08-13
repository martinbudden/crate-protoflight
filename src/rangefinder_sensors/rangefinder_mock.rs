use crate::rangefinder_sensors::{RangefinderMessage, rangefinder::RangefinderDevice};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangefinderMock {}

impl Default for RangefinderMock {
    fn default() -> Self {
        Self::new()
    }
}

impl RangefinderMock {
    pub const fn new() -> Self {
        Self {}
    }
}

impl RangefinderDevice for RangefinderMock {
    fn message(&self) -> RangefinderMessage {
        RangefinderMessage::default()
    }
}
