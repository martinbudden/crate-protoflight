use embassy_time::Instant;

use super::{BarometerI2cError, SharedI2cBus};
use embedded_hal::i2c::I2c;

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
        let mut bus = self.bus.lock().await;
        let mut value = [0u8; 1];
        bus.write_read(self.address, &[register], &mut value).map_err(BarometerI2cError::I2c)?;
        Ok(value[0])
        // self.bus.read_reg(self.address, register).await.map_err(BarometerI2cError::I2c)
    }

    pub async fn write_register(&self, register: u8, value: u8) -> Result<(), BarometerI2cError> {
        let mut bus = self.bus.lock().await;
        bus.write(self.address, &[register, value]).map_err(BarometerI2cError::I2c)
        //self.bus.write_reg(self.address, register, value).await.map_err(BarometerI2cError::I2c)
    }

    pub async fn read_registers<const N: usize>(&self, register: u8) -> Result<[u8; N], BarometerI2cError> {
        let mut bus = self.bus.lock().await;
        let mut value = [0u8; N];
        bus.write_read(self.address, &[register], &mut value).map_err(BarometerI2cError::I2c)?;

        Ok(value)
        //self.bus.read_regs::<N>(self.address, register).await.map_err(BarometerI2cError::I2c)
    }
}
