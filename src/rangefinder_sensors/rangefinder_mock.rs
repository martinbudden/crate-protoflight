use crate::rangefinder_sensors::{RangefinderMessage, rangefinder::RxRangefinder};

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

impl RxRangefinder for RangefinderMock {
    fn message(&self) -> RangefinderMessage {
        RangefinderMessage::default()
    }
}
