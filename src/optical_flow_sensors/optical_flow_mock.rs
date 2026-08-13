use crate::optical_flow_sensors::{OpticalFlowMessage, optical_flow::RxOpticalFlow};

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

impl RxOpticalFlow for OpticalFlowMock {
    fn message(&self) -> OpticalFlowMessage {
        OpticalFlowMessage::default()
    }
}
