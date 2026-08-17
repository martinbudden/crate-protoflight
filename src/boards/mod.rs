#![doc = include_str!("README.md")]

pub mod airb_omnibus_f4;
pub mod board;
pub mod madflight_fc3;
pub mod matek_f405_wte;
pub mod mock_uart;
mod platform_rp2350;
mod platform_std;
mod platform_stm32;
pub mod rpi_pico2;
pub mod sp_racing_f4_evo;
pub mod speedybee_f405_v4;
pub mod std;

pub use board::{BoardInit, ImuContext};

#[cfg(feature = "std")]
pub use {
    platform_std::{GpsUartRx, GpsUartTx, I2cDeviceBlocking},
    std::{BoardImu, board_hardware},
};

#[cfg(feature = "rp2350")]
pub use platform_rp2350::{I2cDeviceBlocking, SharedI2cBus};

#[cfg(feature = "stm32")]
pub use platform_stm32::{GpsUartRx, GpsUartTx, I2cDeviceBlocking};

#[cfg(feature = "rpi_pico2")]
pub use boards::rpi_pico2::{BoardImu, board_hardware};

#[cfg(feature = "madflight_fc3")]
pub use madflight_fc3::{BoardImu, board_hardware};

#[cfg(feature = "speedybee_f405_v4")]
pub use speedybee_f405_v4::{BoardImu, board_hardware, start_realtime_executor};

#[cfg(feature = "sp_racing_f4_evo")]
pub use sp_racing_f4_evo::{BoardImu, board_hardware};

#[cfg(feature = "matek_f405_wte")]
pub use matek_f405_wte::{BoardImu, board_hardware};

#[cfg(feature = "airb_omnibus_f4")]
pub use airb_omnibus_f4::{BoardImu, board_hardware};
