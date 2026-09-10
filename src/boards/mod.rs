#![doc = include_str!("README.md")]

mod platform_host;
mod platform_rp2350;
mod platform_stm32;

mod board;
mod mock_uart;

mod airb_omnibus_f4;
mod host;
mod madflight_fc3;
mod matek_f405_wte;
mod rpi_pico2;
mod sp_racing_f4_evo;
mod speedybee_f405_v4;

pub use board::BoardInit;

#[cfg(feature = "host")]
pub use {
    host::{BoardImu, board_hardware},
    platform_host::{GpsUartRx, GpsUartTx, I2cDeviceBlocking},
};

#[cfg(any(feature = "rp235xa", feature = "rp235xb"))]
pub use platform_rp2350::{I2cDeviceBlocking, SharedI2cBus};

#[allow(unused)]
#[cfg(feature = "stm32")]
pub use platform_stm32::{GpsUartRx, GpsUartTx, I2cDeviceBlocking};

#[cfg(feature = "rpi_pico2")]
pub use rpi_pico2::{BoardImu, board_hardware};

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
