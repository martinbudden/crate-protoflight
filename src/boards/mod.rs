#![doc = include_str!("README.md")]

mod platform_host;
mod platform_rp;
mod platform_stm32;

mod board;
mod mock_uart;

pub mod targets;

pub use board::BoardInit;

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
