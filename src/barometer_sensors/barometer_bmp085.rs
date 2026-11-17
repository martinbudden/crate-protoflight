#![cfg(feature = "barometer")]

use embassy_time::Instant;

use crate::{
    barometer_sensors::{BarometerI2cError, BarometerMessage},
    i2c_bus::{SharedI2cBus, SharedI2cExt},
};

struct Reg;

impl Reg {
    const _DIG_T1: u8 = 0x88;
    const _DIG_T2: u8 = 0x8A;
    const _DIG_T3: u8 = 0x8C;
    const _DIG_P1: u8 = 0x8E;
    const _DIG_P2: u8 = 0x90;
    const _DIG_P3: u8 = 0x92;
    const _DIG_P4: u8 = 0x94;
    const _DIG_P5: u8 = 0x96;
    const _DIG_P6: u8 = 0x98;
    const _DIG_P7: u8 = 0x9A;
    const _DIG_P8: u8 = 0x9C;
    const _DIG_P9: u8 = 0x9E;
    const CHIPID: u8 = 0xD0;
    const _VERSION: u8 = 0xD1;
    const _SOFTRESET: u8 = 0xE0;
    const _CAL26: u8 = 0xE1; // R calibration:u8 = 0xE1-0xF0
    const _STATUS: u8 = 0xF3;
    const _CONTROL: u8 = 0xF4;
    const _CONFIG: u8 = 0xF5;
    const PRESSURE_MSB: u8 = 0xF7;
    const _PRESSURE_LSB: u8 = 0xF8;
    const _PRESSURE_XLSB: u8 = 0xF9;
    const TEMPERATURE_MSB: u8 = 0xFA;
    const _TEMPERATURE_LSB: u8 = 0xFB;
    const _TEMPERATURE_XLSB: u8 = 0xFC;
}

pub struct BarometerBmp085 {
    pub bus: &'static SharedI2cBus,
    address: u8,
    timestamp_us: u64,
    temperature_calibration: TemperatureCalibration,
    temperature_fine: i32,
    temperature_celsius: f32,
    pressure_calibration: PressureCalibration,
    pressure_pascals: f32,
    pressure_at_reference_altitude: f32,
}

impl BarometerBmp085 {
    const I2C_ADDRESS: u8 = 0x76;
    const I2C_ADDRESS_ALTERNATIVE: u8 = 0x77;
    const CHIP_ID: u8 = 0x55;

    pub const fn new(bus: &'static SharedI2cBus) -> Self {
        Self {
            bus,
            address: Self::I2C_ADDRESS,
            timestamp_us: 0,
            temperature_calibration: TemperatureCalibration::new(),
            temperature_fine: 0,
            temperature_celsius: 0.0,
            pressure_calibration: PressureCalibration::new(),
            pressure_pascals: 0.0,
            pressure_at_reference_altitude: 0.0,
        }
    }

    /// Set a newly constructed `BarometerBmp085` to use it's alternative I2C address.
    #[allow(unused)]
    #[must_use]
    pub const fn with_alternative_address(mut self) -> Self {
        self.address = Self::I2C_ADDRESS_ALTERNATIVE;
        self
    }
}

impl BarometerBmp085 {
    fn calculate_temperature(&mut self, temperature: MsbLsbXlsb, calibration: TemperatureCalibration) {
        let adc_t: i32 =
            (((u32::from(temperature.msb) << 16) | (u32::from(temperature.lsb) << 8) | u32::from(temperature.xlsb))
                >> 4)
                .cast_signed();
        let vt1 = ((adc_t >> 3) - (calibration.t1 << 1)) * calibration.t2;
        let vt2 = ((((adc_t >> 4) - calibration.t1) * ((adc_t >> 4) - calibration.t1)) >> 12) * calibration.t3;
        self.temperature_fine = (vt1 >> 11) + (vt2 >> 14);
        #[allow(clippy::cast_precision_loss)]
        {
            self.temperature_celsius = ((self.temperature_fine * 5 + 128) >> 8) as f32 / 100.0;
        }
    }

    fn calculate_pressure(&mut self, pressure: MsbLsbXlsb, calibration: PressureCalibration) {
        let mut vp1 = i64::from(self.temperature_fine) - 128_000;
        let mut vp2 = vp1 * vp1 * i64::from(calibration.p6);
        vp2 += (vp1 * i64::from(calibration.p5)) << 17;
        vp2 += i64::from(calibration.p4) << 35;
        vp1 = ((vp1 * vp1 * i64::from(calibration.p3)) >> 8) + ((vp1 * i64::from(calibration.p2)) << 12);
        vp1 = (((1i64 << 47) + vp1) * i64::from(calibration.p1)) >> 33;

        if vp1 == 0 {
            return; // avoid division by zero
        }
        let adc_p: i32 =
            (((u32::from(pressure.msb) << 16) | (u32::from(pressure.lsb) << 8) | u32::from(pressure.xlsb)) >> 4)
                .cast_signed();

        let mut p: i64 = 1_048_576 - i64::from(adc_p);
        p = (((p << 31) - vp2) * 3125) / vp1;
        let vp1 = (i64::from(calibration.p9) * (p >> 13) * (p >> 13)) >> 25;
        let vp2 = (i64::from(calibration.p8) * p) >> 19;

        p = ((p + vp1 + vp2) >> 8) + (i64::from(calibration.p7) << 4);
        #[allow(clippy::cast_precision_loss)]
        {
            self.pressure_pascals = p as f32 / 256.0;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TemperatureCalibration {
    pub t1: i32,
    pub t2: i32,
    pub t3: i32,
}

impl Default for TemperatureCalibration {
    fn default() -> Self {
        Self::new()
    }
}

impl TemperatureCalibration {
    pub const fn new() -> Self {
        Self { t1: 0, t2: 0, t3: 0 }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct PressureCalibration {
    pub p1: u16,
    pub p2: i16,
    pub p3: i16,
    pub p4: i16,
    pub p5: i16,
    pub p6: i16,
    pub p7: i16,
    pub p8: i16,
    pub p9: i16,
}

impl Default for PressureCalibration {
    fn default() -> Self {
        Self::new()
    }
}

impl PressureCalibration {
    pub const fn new() -> Self {
        Self { p1: 0, p2: 0, p3: 0, p4: 0, p5: 0, p6: 0, p7: 0, p8: 0, p9: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MsbLsbXlsb {
    pub msb: u8,
    pub lsb: u8,
    pub xlsb: u8,
}

impl Default for MsbLsbXlsb {
    fn default() -> Self {
        Self::new()
    }
}

impl MsbLsbXlsb {
    pub const fn new() -> Self {
        Self { msb: 0, lsb: 0, xlsb: 0 }
    }
}

impl BarometerBmp085 {
    async fn read_register(&self, register: u8) -> Result<u8, BarometerI2cError> {
        self.bus.read_reg(self.address, register).await.map_err(BarometerI2cError::I2c)
    }

    async fn _write_register(&self, register: u8, value: u8) -> Result<(), BarometerI2cError> {
        self.bus.write_reg(self.address, register, value).await.map_err(BarometerI2cError::I2c)
    }

    async fn read_registers<const N: usize>(&self, register: u8) -> Result<[u8; N], BarometerI2cError> {
        self.bus.read_regs::<N>(self.address, register).await.map_err(BarometerI2cError::I2c)
    }

    pub async fn init(&self) -> Result<u32, BarometerI2cError> {
        let chip_id = self.read_register(Reg::CHIPID).await?;

        if chip_id != Self::CHIP_ID {
            return Err(BarometerI2cError::InvalidChipId { expected: Self::CHIP_ID, actual: chip_id });
        }

        let sample_rate_hz = 40;
        Ok(sample_rate_hz)
    }

    async fn read_raw_pressure(&self) -> Result<MsbLsbXlsb, BarometerI2cError> {
        let data = self.read_registers::<3>(Reg::PRESSURE_MSB).await?;
        Ok(MsbLsbXlsb { msb: data[0], lsb: data[1], xlsb: data[2] })
    }

    async fn read_raw_temperature(&self) -> Result<MsbLsbXlsb, BarometerI2cError> {
        let data = self.read_registers::<3>(Reg::TEMPERATURE_MSB).await?;
        Ok(MsbLsbXlsb { msb: data[0], lsb: data[1], xlsb: data[2] })
    }

    pub async fn make_reading(&mut self) -> Result<(), BarometerI2cError> {
        let pressure = self.read_raw_pressure().await?;
        let temperature = self.read_raw_temperature().await?;
        self.timestamp_us = Instant::now().as_micros();

        self.calculate_temperature(temperature, self.temperature_calibration);
        self.calculate_pressure(pressure, self.pressure_calibration);
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
