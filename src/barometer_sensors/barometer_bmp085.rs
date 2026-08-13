#![cfg(feature = "barometer")]

use crate::barometer_sensors::{BarometerDevice, BarometerMessage};

const _REG_DIG_T1: u8 = 0x88;
const _REG_DIG_T2: u8 = 0x8A;
const _REG_DIG_T3: u8 = 0x8C;
const _REG_DIG_P1: u8 = 0x8E;
const _REG_DIG_P2: u8 = 0x90;
const _REG_DIG_P3: u8 = 0x92;
const _REG_DIG_P4: u8 = 0x94;
const _REG_DIG_P5: u8 = 0x96;
const _REG_DIG_P6: u8 = 0x98;
const _REG_DIG_P7: u8 = 0x9A;
const _REG_DIG_P8: u8 = 0x9C;
const _REG_DIG_P9: u8 = 0x9E;
const _REG_CHIPID: u8 = 0xD0;
const _REG_VERSION: u8 = 0xD1;
const _REG_SOFTRESET: u8 = 0xE0;
const _REG_CAL26: u8 = 0xE1; // R calibration:u8 = 0xE1-0xF0
const _REG_STATUS: u8 = 0xF3;
const _REG_CONTROL: u8 = 0xF4;
const _REG_CONFIG: u8 = 0xF5;
const _REG_PRESSURE_MSB: u8 = 0xF7;
const _REG_PRESSURE_LSB: u8 = 0xF8;
const _REG_PRESSURE_XLSB: u8 = 0xF9;
const _REG_TEMPERATURE_MSB: u8 = 0xFA;
const _REG_TEMPERATURE_LSB: u8 = 0xFB;
const _REG_TEMPERATURE_XLSB: u8 = 0xFC;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarometerBmp085 {
    calibration: CalibrationData,
    temperature_fine: i32,
    temperature_celsius: f32,
    pressure_pascals: f32,
    pressure_at_reference_altitude: f32,
}

impl Default for BarometerBmp085 {
    fn default() -> Self {
        Self::new()
    }
}

impl BarometerBmp085 {
    const I2C_ADDRESS: u8 = 0x76;
    const I2C_ADDRESS_ALTERNATIVE: u8 = 0x77;
    const CHIP_ID: u8 = 0x58;
    const MAX_SPI_FREQUENCY_HZ: u32 = 10_000_000;

    pub const fn new() -> Self {
        Self {
            calibration: CalibrationData::new(),
            temperature_fine: 0,
            temperature_celsius: 0.0,
            pressure_pascals: 0.0,
            pressure_at_reference_altitude: 0.0,
        }
    }
}

impl BarometerBmp085 {
    fn calculate_temperature_and_pressure(&mut self, temperature: MsbLsbXlsb, pressure: MsbLsbXlsb) {
        let adc_t: i32 =
            ((((temperature.msb) << 16) | ((temperature.lsb) << 8) | (temperature.xlsb)) >> 4).cast_signed();
        let vt1 = ((adc_t >> 3) - (self.calibration.t1 << 1)) * self.calibration.t2;
        let vt2 =
            ((((adc_t >> 4) - self.calibration.t1) * ((adc_t >> 4) - self.calibration.t1)) >> 12) * self.calibration.t3;
        self.temperature_fine = (vt1 >> 11) + (vt2 >> 14);
        #[allow(clippy::cast_precision_loss)]
        {
            self.temperature_celsius = ((self.temperature_fine * 5 + 128) >> 8) as f32 / 100.0;
        }

        let mut vp1 = i64::from(self.temperature_fine) - 128_000;
        let mut vp2 = vp1 * vp1 * i64::from(self.calibration.p6);
        vp2 += (vp1 * i64::from(self.calibration.p5)) << 17;
        vp2 += i64::from(self.calibration.p4) << 35;
        vp1 = ((vp1 * vp1 * i64::from(self.calibration.p3)) >> 8) + ((vp1 * i64::from(self.calibration.p2)) << 12);
        vp1 = (((1i64 << 47) + vp1) * i64::from(self.calibration.p1)) >> 33;

        if (vp1 == 0) {
            return; // avoid division by zero
        }
        let adc_p: i32 = ((((pressure.msb) << 16) | ((pressure.lsb) << 8) | (pressure.xlsb)) >> 4).cast_signed();

        let mut p: i64 = 1_048_576 - i64::from(adc_p);
        p = (((p << 31) - vp2) * 3125) / vp1;
        let vp1 = (i64::from(self.calibration.p9) * (p >> 13) * (p >> 13)) >> 25;
        let vp2 = (i64::from(self.calibration.p8) * p) >> 19;

        p = ((p + vp1 + vp2) >> 8) + (i64::from(self.calibration.p7) << 4);
        #[allow(clippy::cast_precision_loss)]
        {
            self.pressure_pascals = p as f32 / 256.0;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CalibrationData {
    pub t1: i32,
    pub t2: i32,
    pub t3: i32,
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

impl Default for CalibrationData {
    fn default() -> Self {
        Self::new()
    }
}

impl CalibrationData {
    pub const fn new() -> Self {
        Self { t1: 0, t2: 0, t3: 0, p1: 0, p2: 0, p3: 0, p4: 0, p5: 0, p6: 0, p7: 0, p8: 0, p9: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MsbLsbXlsb {
    pub msb: u32,
    pub lsb: u32,
    pub xlsb: u32,
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

impl BarometerDevice for BarometerBmp085 {
    async fn init(&mut self) -> Result<u32, ()> {
        // Placeholder: explicitly await an immediately ready inline future
        core::future::poll_fn(|_| core::task::Poll::Ready(())).await;

        Ok(40)
    }

    async fn make_reading(&mut self) {
        // Placeholder: explicitly await an immediately ready inline future
        core::future::poll_fn(|_| core::task::Poll::Ready(())).await;

        //self.bus.write_register(REG_CONTROL, MEASUREMENT_MODE);
        // read together in burst so data is consistent, as specified in datasheet
        // pressure_temperature_data_u pt;
        // self.bus.read_register(REG_PRESSURE_MSB, &pt.data[0], sizeof(pt));

        let temperature = MsbLsbXlsb::default();
        let pressure = MsbLsbXlsb::default();
        self.calculate_temperature_and_pressure(temperature, pressure);
    }

    fn message(&self) -> BarometerMessage {
        let altitude_m =
            BarometerMessage::calculate_altitude_meters(self.pressure_pascals, self.pressure_at_reference_altitude);
        #[allow(clippy::cast_possible_truncation)]
        BarometerMessage {
            altitude_m,
            altitude_m_i32: altitude_m as i32,
            pressure_pascals: self.pressure_pascals,
            temperature_celsius: self.temperature_celsius,
        }
    }
}
