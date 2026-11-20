#![cfg(feature = "barometer")]

use super::{BarometerCommon, BarometerDevice, BarometerI2cError, SharedI2cBus};

use embassy_time::{Duration, Instant, Timer};

#[derive(Clone, Copy)]
pub struct BarometerBmp280 {
    common: BarometerCommon,
    config: Bmp280Config,
    temperature_calibration: TemperatureCalibration,
    pressure_calibration: PressureCalibration,
}

impl BarometerDevice for BarometerBmp280 {
    async fn init(&mut self) -> Result<Duration, BarometerI2cError> {
        BarometerBmp280::init(self).await
    }

    async fn make_reading(&mut self) -> Result<(Instant, f32, f32), BarometerI2cError> {
        BarometerBmp280::make_reading(self).await
    }
}

struct Reg;

impl Reg {
    const CALIBRATION: u8 = 0x88;
    //const _DIG_T1: u8 = 0x88;
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
    const CHIP_ID: u8 = 0xD0;
    const _VERSION: u8 = 0xD1;
    const SOFT_RESET: u8 = 0xE0;
    const _CAL26: u8 = 0xE1; // R calibration:u8 = 0xE1-0xF0
    const STATUS: u8 = 0xF3;
    const CTRL_MEAS: u8 = 0xF4;
    const CONFIG: u8 = 0xF5;
    const PRESSURE_MSB: u8 = 0xF7;
    const _PRESSURE_LSB: u8 = 0xF8;
    const _PRESSURE_XLSB: u8 = 0xF9;
    const _TEMPERATURE_MSB: u8 = 0xFA;
    const _TEMPERATURE_LSB: u8 = 0xFB;
    const _TEMPERATURE_XLSB: u8 = 0xFC;
}

#[allow(unused)]
impl BarometerBmp280 {
    const I2C_ADDRESS: u8 = 0x76;
    const I2C_ADDRESS_ALTERNATIVE: u8 = 0x77;
    const CHIP_ID: u8 = 0x58;

    pub const fn new(bus: &'static SharedI2cBus) -> Self {
        Self {
            common: BarometerCommon::new(bus, Self::I2C_ADDRESS),

            config: Bmp280Config::new(),

            temperature_calibration: TemperatureCalibration::new(),
            pressure_calibration: PressureCalibration::new(),
        }
    }

    #[must_use]
    pub const fn with_alternative_address(mut self) -> Self {
        self.common.address = Self::I2C_ADDRESS_ALTERNATIVE;
        self
    }

    #[must_use]
    pub const fn with_temperature_oversampling(mut self, temperature_oversampling: Bmp280Oversampling) -> Self {
        self.config.temperature_oversampling = temperature_oversampling;
        self
    }

    #[must_use]
    pub const fn with_config(mut self, config: Bmp280Config) -> Self {
        self.config = config;
        self
    }

    #[must_use]
    pub const fn with_pressure_oversampling(mut self, pressure_oversampling: Bmp280Oversampling) -> Self {
        self.config.pressure_oversampling = pressure_oversampling;
        self
    }

    #[must_use]
    pub const fn with_standby_duration(mut self, standby_duration: Bmp280StandbyDuration) -> Self {
        self.config.standby_duration = standby_duration;
        self
    }

    #[must_use]
    pub const fn with_filter(mut self, filter: Bmp280Filter) -> Self {
        self.config.filter = filter;
        self
    }

    #[must_use]
    pub const fn with_mode(mut self, mode: Bmp280OperatingMode) -> Self {
        self.config.mode = mode;
        self
    }

    pub async fn init(&mut self) -> Result<Duration, BarometerI2cError> {
        let chip_id = self.common.read_register(Reg::CHIP_ID).await?;

        if chip_id != Self::CHIP_ID {
            return Err(BarometerI2cError::InvalidChipId { expected: Self::CHIP_ID, actual: chip_id });
        }

        self.soft_reset().await?;
        self.read_calibration().await?;
        self.configure().await?;

        Ok(Duration::from_micros(u64::from(self.config.measurement_interval_us())))
    }

    async fn soft_reset(&self) -> Result<(), BarometerI2cError> {
        const RESET_TIMEOUT_MS: u32 = 20;
        const RESET_POLL_INTERVAL_MS: u64 = 1;

        self.common.write_register(Reg::SOFT_RESET, 0xB6).await?;

        // Allow the device to reset and reload calibration data.
        Timer::after(Duration::from_millis(2)).await;

        // STATUS.im_update is bit 0.
        // Wait for NVM calibration update to finish.
        for _ in 0..RESET_TIMEOUT_MS {
            let status = self.common.read_register(Reg::STATUS).await?;

            if status & 0x01 == 0 {
                return Ok(());
            }

            Timer::after(Duration::from_millis(RESET_POLL_INTERVAL_MS)).await;
        }
        Err(BarometerI2cError::InvalidData)
    }

    async fn wait_until_idle(&self) -> Result<(), BarometerI2cError> {
        const TIMEOUT_MS: u32 = 100;
        for _ in 0..TIMEOUT_MS {
            let status = self.common.read_register(Reg::STATUS).await?;
            if status & 0x08 == 0 {
                return Ok(());
            }
            Timer::after(Duration::from_millis(1)).await;
        }
        Err(BarometerI2cError::IdleTimeout)
    }

    async fn configure(&self) -> Result<(), BarometerI2cError> {
        // Put sensor into sleep mode before changing CONFIG.
        const CTRL_MEAS_SLEEP: u8 = 0b00;
        self.common.write_register(Reg::CTRL_MEAS, CTRL_MEAS_SLEEP).await?;
        self.wait_until_idle().await?;

        // CONFIG register:
        // [7:5] standby duration
        // [4:2] IIR filter
        // [1]   reserved
        // [0]   SPI 3-wire enable
        let config_reg =
            (self.config.standby_duration.register_value() << 5) | (self.config.filter.register_value() << 2);
        self.common.write_register(Reg::CONFIG, config_reg).await?;
        // CTRL_MEAS register:
        // [7:5] temperature oversampling
        // [4:2] pressure oversampling
        // [1:0] operating mode
        let mode = match self.config.mode {
            Bmp280OperatingMode::Forced => Bmp280OperatingMode::Sleep,
            other => other,
        };
        let ctrl_meas = (self.config.temperature_oversampling.register_value() << 5)
            | (self.config.pressure_oversampling.register_value() << 2)
            | mode.register_value();
        self.common.write_register(Reg::CTRL_MEAS, ctrl_meas).await?;
        Ok(())
    }

    async fn read_calibration(&mut self) -> Result<(), BarometerI2cError> {
        let data = self.common.read_registers::<24>(Reg::CALIBRATION).await?;

        let decode_unsigned = |index: usize| u16::from_le_bytes([data[index], data[index + 1]]);
        let decode_signed = |index: usize| i16::from_le_bytes([data[index], data[index + 1]]);

        self.temperature_calibration =
            TemperatureCalibration { t1: decode_unsigned(0), t2: decode_signed(2), t3: decode_signed(4) };

        self.pressure_calibration = PressureCalibration {
            p1: decode_unsigned(6),
            p2: decode_signed(8),
            p3: decode_signed(10),
            p4: decode_signed(12),
            p5: decode_signed(14),
            p6: decode_signed(16),
            p7: decode_signed(18),
            p8: decode_signed(20),
            p9: decode_signed(22),
        };
        if self.temperature_calibration.t1 == 0 || self.pressure_calibration.p1 == 0 {
            return Err(BarometerI2cError::InvalidCalibration);
        }

        Ok(())
    }

    #[inline]
    fn decode_raw(data: [u8; 6]) -> (u32, u32) {
        let pressure_raw = (u32::from(data[0]) << 12) | (u32::from(data[1]) << 4) | (u32::from(data[2]) >> 4);
        let temperature_raw = (u32::from(data[3]) << 12) | (u32::from(data[4]) << 4) | (u32::from(data[5]) >> 4);

        (temperature_raw, pressure_raw)
    }

    // Returns `(temperature_raw, pressure_raw)`
    async fn read_raw_measurements(&mut self) -> Result<(u32, u32), BarometerI2cError> {
        match self.config.mode {
            Bmp280OperatingMode::Sleep => {
                return Err(BarometerI2cError::InSleepMode);
            }
            Bmp280OperatingMode::Forced => {
                self.trigger_forced_measurement().await?;
            }
            Bmp280OperatingMode::Normal => {
                self.wait_until_idle().await?;
            }
        }
        // Read all pressure and temperature registers in one transaction.
        let data = self.common.read_registers::<6>(Reg::PRESSURE_MSB).await?;
        self.common.timestamp = Instant::now();

        Ok(Self::decode_raw(data))
    }

    fn calculate_temperature(&self, temperature_reading: u32) -> Result<(f32, i32), BarometerI2cError> {
        if self.config.temperature_oversampling == Bmp280Oversampling::Skipped {
            return Err(BarometerI2cError::TemperatureOversamplingNotSet);
        }
        let temperature = temperature_reading.cast_signed();

        let calibration = self.temperature_calibration;

        let t1 = i32::from(calibration.t1);
        let t2 = i32::from(calibration.t2);
        let t3 = i32::from(calibration.t3);

        let var1 = (temperature >> 3) - (t1 << 1);
        let var2 = (temperature >> 4) - t1;
        let vt1 = var1 * t2;
        let vt2 = ((var2 * var2) >> 12) * t3;

        let temperature_fine = (vt1 >> 11) + (vt2 >> 14);
        #[allow(clippy::cast_precision_loss)]
        let temperature_celsius = ((temperature_fine * 5 + 128) >> 8) as f32 / 100.0;
        Ok((temperature_celsius, temperature_fine))
    }

    fn calculate_pressure(&self, pressure_reading: u32, temperature_fine: i32) -> Result<(f32), BarometerI2cError> {
        if self.config.pressure_oversampling == Bmp280Oversampling::Skipped {
            return Err(BarometerI2cError::PressureOversamplingNotSet);
        }

        let calibration = self.pressure_calibration;

        let mut vp1 = i64::from(temperature_fine) - 128_000;

        let mut vp2 = vp1 * vp1 * i64::from(calibration.p6);
        vp2 += (vp1 * i64::from(calibration.p5)) << 17;
        vp2 += i64::from(calibration.p4) << 35;
        vp1 = ((vp1 * vp1 * i64::from(calibration.p3)) >> 8) + ((vp1 * i64::from(calibration.p2)) << 12);
        vp1 = (((1_i64 << 47) + vp1) * i64::from(calibration.p1)) >> 33;

        if vp1 == 0 {
            return Err(BarometerI2cError::InvalidData);
        }

        let mut p = 1_048_576_i64 - i64::from(pressure_reading);
        p = (((p << 31) - vp2) * 3125) / vp1;
        let vp1 = (i64::from(calibration.p9) * (p >> 13) * (p >> 13)) >> 25;
        let vp2 = (i64::from(calibration.p8) * p) >> 19;

        p = ((p + vp1 + vp2) >> 8) + (i64::from(calibration.p7) << 4);
        #[allow(clippy::cast_precision_loss)]
        let pressure_pascals = p as f32 / 256.0;

        if !pressure_pascals.is_finite() || pressure_pascals <= 0.0 {
            return Err(BarometerI2cError::InvalidData);
        }

        Ok(pressure_pascals)
    }

    async fn trigger_forced_measurement(&self) -> Result<(), BarometerI2cError> {
        let ctrl_meas = (self.config.temperature_oversampling.register_value() << 5)
            | (self.config.pressure_oversampling.register_value() << 2)
            | Bmp280OperatingMode::Forced.register_value();

        self.common.write_register(Reg::CTRL_MEAS, ctrl_meas).await?;
        Timer::after(Duration::from_micros(u64::from(self.config.measurement_duration_us()))).await;

        // Confirm that the measurement has finished.
        self.wait_until_idle().await?;

        Ok(())
    }

    pub async fn make_reading(&mut self) -> Result<(Instant, f32, f32), BarometerI2cError> {
        let (temperature_raw, pressure_raw) = self.read_raw_measurements().await?;

        let (temperature_celsius, temperature_fine) = self.calculate_temperature(temperature_raw)?;
        let pressure_pascals = self.calculate_pressure(pressure_raw, temperature_fine)?;

        Ok((self.common.timestamp, temperature_celsius, pressure_pascals))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TemperatureCalibration {
    t1: u16,
    t2: i16,
    t3: i16,
}

impl TemperatureCalibration {
    const fn new() -> Self {
        Self { t1: 0, t2: 0, t3: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PressureCalibration {
    p1: u16,
    p2: i16,
    p3: i16,
    p4: i16,
    p5: i16,
    p6: i16,
    p7: i16,
    p8: i16,
    p9: i16,
}

impl PressureCalibration {
    const fn new() -> Self {
        Self { p1: 0, p2: 0, p3: 0, p4: 0, p5: 0, p6: 0, p7: 0, p8: 0, p9: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bmp280Config {
    pub temperature_oversampling: Bmp280Oversampling,
    pub pressure_oversampling: Bmp280Oversampling,
    pub standby_duration: Bmp280StandbyDuration,
    pub filter: Bmp280Filter,
    pub mode: Bmp280OperatingMode,
}

impl Default for Bmp280Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Bmp280Config {
    /// Create `BmpConfig` with default values suitable for use as input to a Kalman filter, ie no filtering and low oversampling.
    pub const fn new() -> Self {
        Self {
            temperature_oversampling: Bmp280Oversampling::X1,
            pressure_oversampling: Bmp280Oversampling::X4,
            standby_duration: Bmp280StandbyDuration::Ms0_5,
            filter: Bmp280Filter::Off,
            mode: Bmp280OperatingMode::Forced,
        }
    }
}

impl Bmp280Config {
    pub fn measurement_duration_us(self) -> u32 {
        let t_multiplier = self.temperature_oversampling.multiplier();
        let p_multiplier = self.pressure_oversampling.multiplier();
        1_250 + 2_300 * t_multiplier + 2_300 * p_multiplier + if p_multiplier > 0 { 575 } else { 0 }
    }

    pub fn measurement_interval_us(self) -> u32 {
        match self.mode {
            Bmp280OperatingMode::Sleep => 0,
            Bmp280OperatingMode::Forced => self.measurement_duration_us(),
            Bmp280OperatingMode::Normal => self.measurement_duration_us() + self.standby_duration.as_microseconds(),
        }
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Bmp280Oversampling {
    Skipped,
    #[default]
    X1,
    X2,
    X4,
    X8,
    X16,
}

impl Bmp280Oversampling {
    const fn multiplier(self) -> u32 {
        match self {
            Self::Skipped => 0,
            Self::X1 => 1,
            Self::X2 => 2,
            Self::X4 => 4,
            Self::X8 => 8,
            Self::X16 => 16,
        }
    }
    const fn register_value(self) -> u8 {
        match self {
            Self::Skipped => 0b000,
            Self::X1 => 0b001,
            Self::X2 => 0b010,
            Self::X4 => 0b011,
            Self::X8 => 0b100,
            Self::X16 => 0b101,
        }
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Bmp280StandbyDuration {
    #[default]
    Ms0_5,
    Ms62_5,
    Ms125,
    Ms250,
    Ms500,
    Ms1000,
    Ms2000,
    Ms4000,
}

impl Bmp280StandbyDuration {
    pub const fn as_microseconds(self) -> u32 {
        match self {
            Self::Ms0_5 => 500,
            Self::Ms62_5 => 62_500,
            Self::Ms125 => 125_000,
            Self::Ms250 => 250_000,
            Self::Ms500 => 500_000,
            Self::Ms1000 => 1_000_000,
            Self::Ms2000 => 2_000_000,
            Self::Ms4000 => 4_000_000,
        }
    }
    const fn register_value(self) -> u8 {
        match self {
            Self::Ms0_5 => 0b000,
            Self::Ms62_5 => 0b001,
            Self::Ms125 => 0b010,
            Self::Ms250 => 0b011,
            Self::Ms500 => 0b100,
            Self::Ms1000 => 0b101,
            Self::Ms2000 => 0b110,
            Self::Ms4000 => 0b111,
        }
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Bmp280Filter {
    #[default]
    Off,
    X2,
    X4,
    X8,
    X16,
}

impl Bmp280Filter {
    const fn register_value(self) -> u8 {
        match self {
            Self::Off => 0b000,
            Self::X2 => 0b001,
            Self::X4 => 0b010,
            Self::X8 => 0b011,
            Self::X16 => 0b100,
        }
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Bmp280OperatingMode {
    Sleep,
    Forced,
    #[default]
    Normal,
}
impl Bmp280OperatingMode {
    const fn register_value(self) -> u8 {
        match self {
            Self::Sleep => 0b00,
            Self::Forced => 0b01,
            Self::Normal => 0b11,
        }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_sized<T: Sized + Unpin>() {}
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_sized::<BarometerBmp280>();
        is_normal::<TemperatureCalibration>();
        is_normal::<PressureCalibration>();
        is_full::<Bmp280Config>();
        is_full::<Bmp280Oversampling>();
        is_full::<Bmp280StandbyDuration>();
        is_full::<Bmp280Filter>();
        is_full::<Bmp280OperatingMode>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i2c_bus::MockI2c;
    #[allow(unused)]
    use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

    #[test]
    fn measurement_interval() {
        let config = Bmp280Config::new();
        let measurement_interval_us = config.measurement_interval_us();
        assert_eq!(measurement_interval_us, 13325); // 75.04Hz
    }
    #[test]
    fn calculations() {
        // Test taken from example in section 3.12 of BMP280 datasheet.
        use static_cell::StaticCell;
        static I2C_BUS: StaticCell<SharedI2cBus> = StaticCell::new();
        let bus = I2C_BUS.init(Mutex::new(MockI2c::new()));

        let mut barometer = BarometerBmp280::new(bus);
        barometer.temperature_calibration = TemperatureCalibration { t1: 27504, t2: 26435, t3: -1000 };
        let result = barometer.calculate_temperature(519888);
        assert!(result.is_ok());
        if let Ok((temperature_celsius, temperature_fine)) = result {
            assert_eq!(temperature_celsius, 25.08);
            assert_eq!(temperature_fine, 128_422);
        }

        barometer.pressure_calibration = PressureCalibration {
            p1: 36477,
            p2: -10685,
            p3: 3024,
            p4: 2855,
            p5: 140,
            p6: -7,
            p7: 15500,
            p8: -14600,
            p9: 6000,
        };
        let result = barometer.calculate_pressure(415148, 128_422);
        assert!(result.is_ok());
        if let Ok(pressure_pascals) = result {
            assert_eq!(pressure_pascals, 100653.25);
        }
    }
    #[test]
    fn decode_raw() {
        let data = [
            0x65, 0x5A, 0xC0, // Pressure: 415148
            0x7E, 0xED, 0x00, // Temperature: 519888
        ];
        let (temperature, pressure) = BarometerBmp280::decode_raw(data);
        assert_eq!(pressure, 415148);
        assert_eq!(temperature, 519888);
    }
}
