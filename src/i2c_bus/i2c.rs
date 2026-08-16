use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embedded_hal::i2c::{ErrorType, I2c};

use crate::boards::I2cDeviceBlocking;

pub type SharedI2cBus = Mutex<NoopRawMutex, I2cDeviceBlocking>;

pub type I2cError = <I2cDeviceBlocking as embedded_hal::i2c::ErrorType>::Error;

// `Ext` means "extension trait"
// The `Ext` suffix conventionally means:
// "This trait adds convenient methods to an existing type."
/// Extension methods for the shared I2C bus.
pub trait SharedI2cExt {
    type Error;

    async fn write_reg(&self, address: u8, register: u8, value: u8) -> Result<(), Self::Error>;

    async fn read_reg(&self, address: u8, register: u8) -> Result<u8, Self::Error>;

    async fn read_regs<const N: usize>(&self, address: u8, register: u8) -> Result<[u8; N], Self::Error>;
}

impl SharedI2cExt for SharedI2cBus {
    type Error = <I2cDeviceBlocking as ErrorType>::Error;

    async fn write_reg(&self, address: u8, register: u8, value: u8) -> Result<(), Self::Error> {
        let mut bus = self.lock().await;
        bus.write(address, &[register, value])
    }

    async fn read_reg(&self, address: u8, register: u8) -> Result<u8, Self::Error> {
        let mut bus = self.lock().await;

        let mut value = [0u8; 1];

        bus.write_read(address, &[register], &mut value)?;

        Ok(value[0])
    }

    async fn read_regs<const N: usize>(&self, address: u8, register: u8) -> Result<[u8; N], Self::Error> {
        let mut bus = self.lock().await;

        let mut value = [0u8; N];

        bus.write_read(address, &[register], &mut value)?;

        Ok(value)
    }
}
