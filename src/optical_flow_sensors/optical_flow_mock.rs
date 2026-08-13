use crate::optical_flow_sensors::{OpticalFlowMessage, optical_flow::OpticalFlowDevice};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpticalFlowMock {}

impl Default for OpticalFlowMock {
    fn default() -> Self {
        Self::new()
    }
}

impl OpticalFlowMock {
    pub const fn new() -> Self {
        Self {}
    }
}

impl OpticalFlowDevice for OpticalFlowMock {
    fn message(&self) -> OpticalFlowMessage {
        OpticalFlowMessage::default()
    }
}
