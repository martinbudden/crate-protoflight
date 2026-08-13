#![cfg(feature = "optical_flow")]

use crate::optical_flow_sensors::{OpticalFlowMessage, optical_flow::OpticalFlowDevice};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpticalFlowMt {}

impl Default for OpticalFlowMt {
    fn default() -> Self {
        Self::new()
    }
}

impl OpticalFlowMt {
    pub const fn new() -> Self {
        Self {}
    }
}

impl OpticalFlowDevice for OpticalFlowMt {
    fn message(&self) -> OpticalFlowMessage {
        OpticalFlowMessage::default()
    }
}
