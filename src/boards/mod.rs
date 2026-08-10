#![doc = include_str!("README.md")]

pub mod board;
pub mod matek_f405_wte;
pub mod rp2350;
pub mod speedybee_f405_v4;
pub mod std;

pub use board::ImuContext;

#[cfg(feature = "std")]
pub use crate::boards::std::{BoardImu, board_init, imu_context};

#[cfg(feature = "rp2350")]
pub use crate::boards::rp2350::{BoardImu, board_init, imu_context};

#[cfg(feature = "speedybee_f405_v4")]
pub use crate::boards::speedybee_f405_v4::{BoardImu, board_init, imu_context};

#[cfg(feature = "matek_f405_wte")]
pub use crate::boards::matek_f405_wte::{BoardImu, board_init, imu_context};
