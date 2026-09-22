#![doc = include_str!("README.md")]

mod altitude_dual_ring_pid;
mod xy_position_dual_ring_pid;

mod config;
mod mock_multirotor;
mod path_follower;
mod pilot;

pub use config::{AutopilotConfig, PositionHoldConfig};
#[allow(unused)]
pub use mock_multirotor::{MockMultirotorXY, MockMultirotorZ};
#[cfg(feature = "autopilot")]
pub use pilot::Autopilot;
