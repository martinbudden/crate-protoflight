#![cfg(feature = "std")]

use crate::boards::mock_uart::MockUart;

pub type GpsUartRx = MockUart;
pub type GpsUartTx = MockUart;
pub type I2cDeviceBlocking = crate::i2c_bus::MockI2c;
