#![cfg(feature = "barometer")]

use embassy_time::Instant;

use crate::{
    barometer_sensors::{BarometerI2cError, BarometerMessage},
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
    pub bus: &'static SharedI2cBus,
    address: u8,
    timestamp_us: u64,
    temperature_raw: i32,
    temperature_celsius: f32,
    pressure_raw: i32,
    pressure_pascals: f32,
    pressure_at_reference_altitude: f32,
}

impl BarometerDps310 {
    pub const I2C_ADDRESS: u8 = 0x76;
    pub const I2C_ADDRESS_ALTERNATIVE: u8 = 0x77;
    const CHIP_ID_DPS310: u8 = 0x10;
    const CHIP_ID_SPL07_003: u8 = 0x10;
    const _MAX_SPI_FREQUENCY_HZ: u32 = 10_000_000;

    pub const fn new(bus: &'static SharedI2cBus) -> Self {
        Self {
            bus,
            address: Self::I2C_ADDRESS,
            timestamp_us: 0,
            temperature_raw: 0,
            temperature_celsius: 0.0,
            pressure_raw: 0,
            pressure_pascals: 0.0,
            pressure_at_reference_altitude: 0.0,
        }
    }

    /// Set a newly constructed `BarometerDps310` to use it's alternative I2C address.
    #[allow(unused)]
    #[must_use]
    pub const fn with_alternative_address(mut self) -> Self {
        self.address = Self::I2C_ADDRESS_ALTERNATIVE;
        self
    }
}

impl BarometerDps310 {
    async fn read_register(&self, register: u8) -> Result<u8, BarometerI2cError> {
        self.bus.read_reg(self.address, register).await.map_err(BarometerI2cError::I2c)
    }

    async fn write_register(&self, register: u8, value: u8) -> Result<(), BarometerI2cError> {
        self.bus.write_reg(self.address, register, value).await.map_err(BarometerI2cError::I2c)
    }

    async fn read_registers<const N: usize>(&self, register: u8) -> Result<[u8; N], BarometerI2cError> {
        self.bus.read_regs::<N>(self.address, register).await.map_err(BarometerI2cError::I2c)
    }

    pub async fn init(&self) -> Result<u32, BarometerI2cError> {
        let chip_id = self.read_register(Reg::ID).await?;

        if chip_id != Self::CHIP_ID_DPS310 && chip_id != Self::CHIP_ID_SPL07_003 {
            return Err(BarometerI2cError::InvalidChipId { expected: Self::CHIP_ID_DPS310, actual: chip_id });
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

        let sample_rate_hz = 40;
        Ok(sample_rate_hz)
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

    pub async fn make_reading(&mut self) -> Result<(), BarometerI2cError> {
        self.pressure_raw = self.read_raw_pressure().await?;
        self.temperature_raw = self.read_raw_temperature().await?;
        self.timestamp_us = Instant::now().as_micros();
        Ok(())
    }

    pub fn message(&self) -> BarometerMessage {
        let altitude_m =
            BarometerMessage::calculate_altitude_meters(self.pressure_pascals, self.pressure_at_reference_altitude);
        #[allow(clippy::cast_possible_truncation)]
        BarometerMessage {
            timestamp_us: self.timestamp_us,
            altitude_m,
            altitude_m_i32: altitude_m as i32,
            pressure_pascals: self.pressure_pascals,
            temperature_celsius: self.temperature_celsius,
        }
    }
}
