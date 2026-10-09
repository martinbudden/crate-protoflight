#![doc = include_str!("README.md")]

mod platform_esp32s3;
mod platform_host;
mod platform_rp;
mod platform_stm32;

mod board;
mod board_init_error;
mod mock_uart;
mod multicore_executor;
mod no_sdcard;
mod sd_card;

pub mod targets;

pub use board::{BoardHardware, BoardInit};
pub use board_init_error::BoardInitError;

#[allow(unused)]
#[cfg(feature = "multicore")]
pub use multicore_executor::start_core1_executor;

#[allow(unused)]
#[cfg(feature = "sdcard")]
pub use sd_card::{SdCardBlockDevice, SdCardTimeSource, SdCardVolume, StorageError, StorageVolumeError, open_volume};

#[allow(unused)]
#[cfg(not(feature = "sdcard"))]
pub use no_sdcard::SdCardVolume;

#[cfg(feature = "host")]
pub use platform_host::{GpsUartRx, GpsUartTx, I2cDeviceBlocking, RadioUartRx, RadioUartTx};

#[allow(unused)]
#[cfg(any(feature = "rp2040", feature = "rp235xa", feature = "rp235xb"))]
pub use platform_rp::{
    BufferedRadioUartRx, GpsUartRx, GpsUartTx, I2cDeviceBlocking, RadioUartRx, RadioUartTx, SharedI2cBus,
};

#[allow(unused)]
#[cfg(feature = "stm32")]
pub use platform_stm32::{GpsUartRx, GpsUartTx, I2cDeviceBlocking, RadioUartRx, RadioUartTx, SharedI2cBus};

#[allow(unused)]
#[cfg(feature = "esp32s3")]
pub use platform_esp32s3::{GpsUartRx, GpsUartTx, I2cDeviceBlocking, RadioUartRx, RadioUartTx, SharedI2cBus};
