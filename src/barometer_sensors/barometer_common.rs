use embassy_time::Instant;

use super::BarometerI2cError;
use crate::i2c_bus::{SharedI2cBus, SharedI2cExt};

#[derive(Clone, Copy, Debug)]
pub struct BarometerCommon {
    pub(crate) bus: &'static SharedI2cBus,
    pub(crate) address: u8,
    pub(crate) timestamp: Instant,
}

impl BarometerCommon {
    pub const fn new(bus: &'static SharedI2cBus, address: u8) -> Self {
        Self { bus, address, timestamp: Instant::MIN }
    }
    pub async fn read_register(&self, register: u8) -> Result<u8, BarometerI2cError> {
        self.bus.read_reg(self.address, register).await.map_err(BarometerI2cError::I2c)
    }

    pub async fn write_register(&self, register: u8, value: u8) -> Result<(), BarometerI2cError> {
        self.bus.write_reg(self.address, register, value).await.map_err(BarometerI2cError::I2c)
    }

    pub async fn read_registers<const N: usize>(&self, register: u8) -> Result<[u8; N], BarometerI2cError> {
        self.bus.read_regs::<N>(self.address, register).await.map_err(BarometerI2cError::I2c)
    }
}
