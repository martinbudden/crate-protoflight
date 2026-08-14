#![doc = include_str!("README.md")]

pub mod airb_omnibus_f4;
pub mod board;
pub mod madflight_fc3;
pub mod matek_f405_wte;
pub mod mock_i2c;
pub mod mock_uart;
pub mod rpi_pico2;
pub mod sp_racing_f4_evo;
pub mod speedybee_f405_v4;
pub mod std;

pub use board::{BoardInit, ImuContext};

#[cfg(feature = "std")]
pub use crate::boards::std::{BoardImu, board_hardware};

#[cfg(feature = "rpi_pico2")]
pub use crate::boards::rpi_pico2::{BoardImu, board_hardware};

#[cfg(feature = "madflight_fc3")]
pub use crate::boards::madflight_fc3::{BoardImu, board_hardware};

#[cfg(feature = "speedybee_f405_v4")]
pub use crate::boards::speedybee_f405_v4::{BoardImu, board_hardware};

#[cfg(feature = "sp_racing_f4_evo")]
pub use crate::boards::sp_racing_f4_evo::{BoardImu, board_hardware};

#[cfg(feature = "matek_f405_wte")]
pub use crate::boards::matek_f405_wte::{BoardImu, board_hardware};

#[cfg(feature = "airb_omnibus_f4")]
pub use crate::boards::airb_omnibus_f4::{BoardImu, board_hardware};
