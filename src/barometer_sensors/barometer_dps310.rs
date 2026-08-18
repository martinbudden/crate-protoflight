//#![cfg(feature = "barometer")]
#![allow(unused)]
use crate::{
    barometer_sensors::{BarometerI2cError, BarometerMessage, barometer::BarometerError},
    i2c_bus::{SharedI2cBus, SharedI2cExt},
};

struct Reg;

impl Reg {
    const PRS_B2: u8 = 0x00;
    //const PRS_B2
    const _PSR_B1: u8 = 0x01;
    const _PSR_B0: u8 = 0x02;
    const TMP_B2: u8 = 0x03;
    const _TMP_B1: u8 = 0x04;
    const _TMP_B0: u8 = 0x05;
    const _PRS_CFG: u8 = 0x06;
    const _TMP_CFG: u8 = 0x07;
    const _MEAS_CFG: u8 = 0x08;
    const _CFG_REG: u8 = 0x09;

    const RESET: u8 = 0x0C;
    const ID: u8 = 0x0D;

    const _COEF: u8 = 0x10;
    const _COEF_SRCE: u8 = 0x28;
}

pub struct BarometerDps310 {
    pub i2c_bus: &'static SharedI2cBus,
}

impl BarometerDps310 {
    const I2C_ADDRESS: u8 = 0x76;
    const CHIP_ID: u8 = 0x11;
    const MAX_SPI_FREQUENCY_HZ: u32 = 10_000_000;

    pub const fn new(i2c_bus: &'static SharedI2cBus) -> Self {
        Self { i2c_bus }
    }
}

/*impl BarometerDps310 {
    async fn read_register(&self, register: u8) -> Result<u8, ()> {
        let mut data = [0u8; 1];

        let mut i2c = self.i2c_bus.lock().await;
        i2c.blocking_write_read(Self::I2C_ADDRESS, &[register], &mut data);

        Ok(data[0])
    }

    async fn read_registers(&self, register: u8, data: &mut [u8]) -> Result<(), ()> {
        let mut i2c = self.i2c_bus.lock().await;
        i2c.blocking_write_read(Self::I2C_ADDRESS, &[register], data);

        Ok(())
    }

    async fn write_register(&self, register: u8, value: u8) -> Result<(), ()> {
        let mut i2c = self.i2c_bus.lock().await;
        i2c.blocking_write(Self::I2C_ADDRESS, &[register, value]);

        Ok(())
    }
}

impl BarometerDevice for BarometerDps310 {
    async fn init(&mut self) -> Result<u32, ()> {
        // Placeholder: explicitly await an immediately ready inline future
        core::future::poll_fn(|_| core::task::Poll::Ready(())).await;

        Ok(0)
    }

    async fn make_reading(&mut self) {
        // Placeholder: explicitly await an immediately ready inline future
        core::future::poll_fn(|_| core::task::Poll::Ready(())).await;
        _ = self;
    }
    fn message(&self) -> BarometerMessage {
        BarometerMessage::default()
    }
}*/

impl BarometerDps310 {
    async fn read_register(&self, register: u8) -> Result<u8, BarometerI2cError> {
        self.i2c_bus.read_reg(Self::I2C_ADDRESS, register).await.map_err(BarometerError::I2c)
    }

    async fn write_register(&self, register: u8, value: u8) -> Result<(), BarometerI2cError> {
        self.i2c_bus.write_reg(Self::I2C_ADDRESS, register, value).await.map_err(BarometerError::I2c)
    }

    async fn read_registers<const N: usize>(&self, register: u8) -> Result<[u8; N], BarometerI2cError> {
        self.i2c_bus.read_regs::<N>(Self::I2C_ADDRESS, register).await.map_err(BarometerError::I2c)
    }

    pub async fn init(&self) -> Result<u32, BarometerI2cError> {
        let chip_id = self.read_register(Reg::ID).await?;

        if chip_id != Self::CHIP_ID {
            return Err(BarometerError::InvalidChipId { expected: Self::CHIP_ID, actual: chip_id });
        }

        // Reset the DPS310.
        self.write_register(Reg::RESET, 0x89).await?;

        // TODO: wait for the device to complete its reset.

        // Configure pressure and temperature measurements here.
        //
        // For example:
        // self.write_register(Self::REG_PRS_CFG, ...).await?;
        // self.write_register(Self::REG_TMP_CFG, ...).await?;
        // self.write_register(Self::REG_MEAS_CFG, ...).await?;

        Ok(40)
    }

    pub async fn read_raw_pressure(&self) -> Result<i32, BarometerI2cError> {
        let data = self.read_registers::<3>(Reg::PRS_B2).await?;
        let raw = (i32::from(data[0]) << 16) | (i32::from(data[1]) << 8) | i32::from(data[2]);
        Ok(if raw & 0x80_0000 != 0 { raw | !0xFF_FFFF } else { raw })
    }

    pub async fn read_raw_temperature(&self) -> Result<i32, BarometerI2cError> {
        let data = self.read_registers::<3>(Reg::TMP_B2).await?;
        let raw = (i32::from(data[0]) << 16) | (i32::from(data[1]) << 8) | i32::from(data[2]);
        // Signed 24-bit value.
        Ok(if raw & 0x80_0000 != 0 { raw | !0xFF_FFFF } else { raw })
    }

    pub async fn make_reading(&mut self) {
        // Placeholder: explicitly await an immediately ready inline future
        core::future::poll_fn(|_| core::task::Poll::Ready(())).await;
    }

    pub fn message(&self) -> BarometerMessage {
        _ = self;
        BarometerMessage::default()
    }
}
