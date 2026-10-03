#![doc = include_str!("README.md")]

mod dual_ring_pid_horizontal;
mod dual_ring_pid_vertical;

mod config;
mod mock_multirotor;
mod path_follower;
mod pilot;

pub use config::{AutopilotConfig, PositionHoldConfig};
#[allow(unused)]
pub use mock_multirotor::{MockMultirotorXY, MockMultirotorZ};
#[cfg(feature = "autopilot")]
pub use pilot::Autopilot;
