use super::{BarometerCommon, BarometerDevice, BarometerI2cError, SharedI2cBus};

use embassy_time::{Duration, Instant, Timer};

#[derive(Clone, Copy)]
pub struct BarometerDps310 {
    common: BarometerCommon,
    config: Dps310Config,

    temperature_calibration: TemperatureCalibration,
    pressure_calibration: PressureCalibration,

    temperature_scale_factor: f32,
    pressure_scale_factor: f32,

    temperature_sensor_external: bool,
}

impl BarometerDevice for BarometerDps310 {
    async fn init(&mut self) -> Result<Duration, BarometerI2cError> {
        BarometerDps310::init(self).await
    }

    async fn make_reading(&mut self) -> Result<(Instant, f32, f32), BarometerI2cError> {
        BarometerDps310::make_reading(self).await
    }
}

struct Reg;

impl Reg {
    const PSR_B2: u8 = 0x00;
    const _PSR_B1: u8 = 0x01;
    const _PSR_B0: u8 = 0x02;
    const TMP_B2: u8 = 0x03;
    const _TMP_B1: u8 = 0x04;
    const _TMP_B0: u8 = 0x05;

    const PRS_CFG: u8 = 0x06;
    const TMP_CFG: u8 = 0x07;
    const MEAS_CFG: u8 = 0x08;
    const CFG_REG: u8 = 0x09;
    const _INT_STS: u8 = 0x0A;
    const _FIFO_STS: u8 = 0x0B;

    const RESET: u8 = 0x0C;
    const PROD_ID: u8 = 0x0D;

    const COEF: u8 = 0x10; // 0x10 - 0x21
    const COEF_SRCE: u8 = 0x28;
}

#[allow(unused)]
impl BarometerDps310 {
    const I2C_ADDRESS: u8 = 0x77;
    const I2C_ADDRESS_ALTERNATIVE: u8 = 0x76;
    const CHIP_ID: u8 = 0x00;

    pub const fn new(bus: &'static SharedI2cBus) -> Self {
        Self {
            common: BarometerCommon::new(bus, Self::I2C_ADDRESS),
            config: Dps310Config::new(),

            temperature_calibration: TemperatureCalibration::new(),
            pressure_calibration: PressureCalibration::new(),

            temperature_scale_factor: Dps310Oversampling::X1.scale_factor(),
            pressure_scale_factor: Dps310Oversampling::X1.scale_factor(),

            temperature_sensor_external: false,
        }
    }

    #[must_use]
    pub const fn with_alternative_address(mut self) -> Self {
        self.common.address = Self::I2C_ADDRESS_ALTERNATIVE;
        self
    }

    #[must_use]
    pub const fn with_config(mut self, config: Dps310Config) -> Self {
        self.config = config;
        self
    }

    #[must_use]
    pub const fn with_temperature_oversampling(mut self, oversampling: Dps310Oversampling) -> Self {
        self.config.temperature_oversampling = oversampling;
        self
    }

    #[must_use]
    pub const fn with_pressure_oversampling(mut self, oversampling: Dps310Oversampling) -> Self {
        self.config.pressure_oversampling = oversampling;
        self
    }

    #[must_use]
    pub const fn with_pressure_measurement_rate(mut self, rate: Dps310MeasurementRate) -> Self {
        self.config.pressure_measurement_rate = rate;
        self
    }

    #[must_use]
    pub const fn with_temperature_measurement_rate(mut self, rate: Dps310MeasurementRate) -> Self {
        self.config.temperature_measurement_rate = rate;
        self
    }

    pub async fn init(&mut self) -> Result<Duration, BarometerI2cError> {
        let chip_id = self.common.read_register(Reg::PROD_ID).await?;
        let product_id = chip_id & 0x0F;

        if product_id != Self::CHIP_ID {
            return Err(BarometerI2cError::InvalidChipId { expected: Self::CHIP_ID, actual: product_id });
        }

        self.reset().await?;
        self.read_calibration().await?;
        self.configure().await?;

        Ok(Duration::from_micros(u64::from(self.config.measurement_time_us())))
    }

    async fn reset(&self) -> Result<(), BarometerI2cError> {
        const RESET_TIMEOUT_MS: u32 = 60;
        const RESET_POLL_INTERVAL_MS: u64 = 1;
        const STATUS_COEFFICIENTS_READY: u8 = 1 << 7;
        const STATUS_SENSOR_READY: u8 = 1 << 6;

        self.common.write_register(Reg::RESET, 0x09).await?;
        for _ in 0..RESET_TIMEOUT_MS {
            let status = self.common.read_register(Reg::MEAS_CFG).await?;
            if status & STATUS_SENSOR_READY != 0 && status & STATUS_COEFFICIENTS_READY != 0 {
                return Ok(());
            }
            Timer::after(Duration::from_millis(RESET_POLL_INTERVAL_MS)).await;
        }
        Err(BarometerI2cError::InvalidData)
    }

    async fn read_calibration(&mut self) -> Result<(), BarometerI2cError> {
        let data = self.common.read_registers::<18>(Reg::COEF).await?;

        let (temperature_calibration, pressure_calibration) = Self::decode_calibration(data);

        self.temperature_calibration = temperature_calibration;
        self.pressure_calibration = pressure_calibration;

        let coefficient_source = self.common.read_register(Reg::COEF_SRCE).await?;

        self.temperature_sensor_external = coefficient_source & 0x80 != 0;

        Ok(())
    }

    fn decode_calibration(data: [u8; 18]) -> (TemperatureCalibration, PressureCalibration) {
        let c0 = (u32::from(data[0]) << 4) | (u32::from(data[1]) >> 4);
        let c1 = ((u32::from(data[1]) & 0x0F) << 8) | u32::from(data[2]);
        let c00 = (u32::from(data[3]) << 12) | (u32::from(data[4]) << 4) | (u32::from(data[5]) >> 4);
        let c10 = ((u32::from(data[5]) & 0x0F) << 16) | (u32::from(data[6]) << 8) | u32::from(data[7]);

        let decode_i16 = |index: usize| -> i32 { i32::from(i16::from_be_bytes([data[index], data[index + 1]])) };

        #[allow(clippy::cast_possible_truncation)]
        let temperature =
            TemperatureCalibration { c0: Self::sign_extend(c0, 12) as i16, c1: Self::sign_extend(c1, 12) as i16 };

        let pressure = PressureCalibration {
            c00: Self::sign_extend(c00, 20),
            c10: Self::sign_extend(c10, 20),
            c01: decode_i16(8),
            c11: decode_i16(10),
            c20: decode_i16(12),
            c21: decode_i16(14),
            c30: decode_i16(16),
        };

        (temperature, pressure)
    }

    async fn configure(&mut self) -> Result<(), BarometerI2cError> {
        const PRESSURE_SHIFT_ENABLE: u8 = 1 << 2;
        const TEMPERATURE_SHIFT_ENABLE: u8 = 1 << 3;
        const TEMPERATURE_EXTERNAL_ENABLE: u8 = 1 << 7;

        let pressure_osr = self.config.pressure_oversampling;
        let temperature_osr = self.config.temperature_oversampling;

        self.common.write_register(Reg::PRS_CFG, pressure_osr.register_value()).await?;

        let temperature_config = (if self.temperature_sensor_external { TEMPERATURE_EXTERNAL_ENABLE } else { 0 })
            | temperature_osr.register_value();

        self.common.write_register(Reg::TMP_CFG, temperature_config).await?;

        let mut config_reg = 0;

        if pressure_osr.requires_shift() {
            config_reg |= PRESSURE_SHIFT_ENABLE;
        }

        if temperature_osr.requires_shift() {
            config_reg |= TEMPERATURE_SHIFT_ENABLE;
        }

        self.common.write_register(Reg::CFG_REG, config_reg).await?;
        self.temperature_scale_factor = temperature_osr.scale_factor();
        self.pressure_scale_factor = pressure_osr.scale_factor();

        Ok(())
    }

    #[inline]
    async fn wait_for_measurement(&self, ready_mask: u8, timeout_ms: u32) -> Result<(), BarometerI2cError> {
        for _ in 0..timeout_ms {
            let status = self.common.read_register(Reg::MEAS_CFG).await?;
            if status & ready_mask != 0 {
                return Ok(());
            }
            Timer::after(Duration::from_millis(1)).await;
        }
        Err(BarometerI2cError::InvalidData)
    }

    #[inline]
    async fn read_raw_measurements(&mut self) -> Result<(i32, i32), BarometerI2cError> {
        const COMMAND_TEMPERATURE: u8 = 0b010;
        const COMMAND_PRESSURE: u8 = 0b001;

        const STATUS_TEMPERATURE_READY: u8 = 1 << 5;
        const STATUS_PRESSURE_READY: u8 = 1 << 4;

        // Temperature measurement.
        let temperature_timeout = self.config.temperature_oversampling.measurement_time_ms() + 10;
        self.common.write_register(Reg::MEAS_CFG, COMMAND_TEMPERATURE).await?;
        self.wait_for_measurement(STATUS_TEMPERATURE_READY, temperature_timeout).await?;
        let temperature_data = self.common.read_registers::<3>(Reg::TMP_B2).await?;

        // Pressure measurement.
        let pressure_timeout = self.config.pressure_oversampling.measurement_time_ms() + 10;
        self.common.write_register(Reg::MEAS_CFG, COMMAND_PRESSURE).await?;
        self.wait_for_measurement(STATUS_PRESSURE_READY, pressure_timeout).await?;
        self.common.timestamp = Instant::now();

        let pressure_data = self.common.read_registers::<3>(Reg::PSR_B2).await?;

        let temperature = Self::decode_raw_measurement(temperature_data);
        let pressure = Self::decode_raw_measurement(pressure_data);

        Ok((temperature, pressure))
    }

    #[allow(clippy::cast_possible_wrap)]
    #[inline]
    fn sign_extend(value: u32, bits: u32) -> i32 {
        let shift = 32 - bits;
        ((value << shift) as i32) >> shift
    }

    #[inline]
    fn decode_raw_measurement(data: [u8; 3]) -> i32 {
        let value = u32::from_be_bytes([0, data[0], data[1], data[2]]);
        Self::sign_extend(value, 24)
    }

    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn calculate_temperature_and_pressure(&self, temperature_raw: i32, pressure_raw: i32) -> (f32, f32) {
        let t_scaled = temperature_raw as f32 / self.temperature_scale_factor;
        let tc = self.temperature_calibration;

        let temperature = f32::from(tc.c0) * 0.5 + f32::from(tc.c1) * t_scaled;

        let p_scaled = pressure_raw as f32 / self.pressure_scale_factor;
        let pc = self.pressure_calibration;

        let pressure = pc.c00 as f32
            + p_scaled * (pc.c10 as f32 + p_scaled * (pc.c20 as f32 + p_scaled * pc.c30 as f32))
            + t_scaled * (pc.c01 as f32 + p_scaled * (pc.c11 as f32 + p_scaled * pc.c21 as f32));
        (temperature, pressure)
    }

    pub async fn make_reading(&mut self) -> Result<(Instant, f32, f32), BarometerI2cError> {
        let (temperature_raw, pressure_raw) = self.read_raw_measurements().await?;

        let (temperature, pressure) = self.calculate_temperature_and_pressure(temperature_raw, pressure_raw);

        if !temperature.is_finite() || !pressure.is_finite() || pressure <= 0.0 {
            return Err(BarometerI2cError::InvalidData);
        }

        Ok((self.common.timestamp, temperature, pressure))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TemperatureCalibration {
    c0: i16,
    c1: i16,
}

impl TemperatureCalibration {
    #[inline]
    const fn new() -> Self {
        Self { c0: 0, c1: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PressureCalibration {
    c00: i32,
    c10: i32,
    c01: i32,
    c11: i32,
    c20: i32,
    c21: i32,
    c30: i32,
}

impl PressureCalibration {
    #[inline]
    const fn new() -> Self {
        Self { c00: 0, c10: 0, c01: 0, c11: 0, c20: 0, c21: 0, c30: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dps310Config {
    pub temperature_oversampling: Dps310Oversampling,
    pub pressure_oversampling: Dps310Oversampling,
    pub temperature_measurement_rate: Dps310MeasurementRate,
    pub pressure_measurement_rate: Dps310MeasurementRate,
}

impl Default for Dps310Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Dps310Config {
    #[inline]
    pub const fn new() -> Self {
        Self {
            temperature_oversampling: Dps310Oversampling::X1,
            pressure_oversampling: Dps310Oversampling::X4,
            temperature_measurement_rate: Dps310MeasurementRate::Hz1,
            pressure_measurement_rate: Dps310MeasurementRate::Hz1,
        }
    }

    #[inline]
    pub fn measurement_time_us(self) -> u32 {
        self.temperature_oversampling.measurement_time_us() + self.pressure_oversampling.measurement_time_us()
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dps310Oversampling {
    #[default]
    X1,
    X2,
    X4,
    X8,
    X16,
    X32,
    X64,
    X128,
}

impl Dps310Oversampling {
    const fn register_value(self) -> u8 {
        match self {
            Self::X1 => 0b000,
            Self::X2 => 0b001,
            Self::X4 => 0b010,
            Self::X8 => 0b011,
            Self::X16 => 0b100,
            Self::X32 => 0b101,
            Self::X64 => 0b110,
            Self::X128 => 0b111,
        }
    }

    const fn requires_shift(self) -> bool {
        matches!(self, Self::X16 | Self::X32 | Self::X64 | Self::X128)
    }

    const fn scale_factor(self) -> f32 {
        match self {
            Self::X1 => 524_288.0,
            Self::X2 => 1_572_864.0,
            Self::X4 => 3_670_016.0,
            Self::X8 => 7_864_320.0,
            Self::X16 => 253_952.0,
            Self::X32 => 516_096.0,
            Self::X64 => 1_040_384.0,
            Self::X128 => 2_088_960.0,
        }
    }

    const fn measurement_time_ms(self) -> u32 {
        match self {
            Self::X1 => 4,
            Self::X2 => 6,
            Self::X4 => 9,
            Self::X8 => 15,
            Self::X16 => 28,
            Self::X32 => 54,
            Self::X64 => 105,
            Self::X128 => 207,
        }
    }

    const fn measurement_time_us(self) -> u32 {
        self.measurement_time_ms() * 1_000
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dps310MeasurementRate {
    #[default]
    Hz1,
    Hz2,
    Hz4,
    Hz8,
    Hz16,
    Hz32,
    Hz64,
    Hz128,
}

#[allow(unused)]
impl Dps310MeasurementRate {
    const fn register_value(self) -> u8 {
        match self {
            Self::Hz1 => 0,
            Self::Hz2 => 1,
            Self::Hz4 => 2,
            Self::Hz8 => 3,
            Self::Hz16 => 4,
            Self::Hz32 => 5,
            Self::Hz64 => 6,
            Self::Hz128 => 7,
        }
    }
    const fn register_bits(self) -> u8 {
        self.register_value() << 4
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
        is_sized::<BarometerDps310>();
        is_normal::<TemperatureCalibration>();
        is_normal::<PressureCalibration>();
        is_full::<Dps310Config>();
        is_full::<Dps310Oversampling>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_extend() {
        assert_eq!(BarometerDps310::sign_extend(0x7FF, 12), 2047);
        assert_eq!(BarometerDps310::sign_extend(0x800, 12), -2048);
        assert_eq!(BarometerDps310::sign_extend(0xFFF, 12), -1);

        assert_eq!(BarometerDps310::sign_extend(0x7FFFF, 20), 524_287);
        assert_eq!(BarometerDps310::sign_extend(0x80000, 20), -524_288);
        assert_eq!(BarometerDps310::sign_extend(0xFFFFF, 20), -1);
    }

    #[test]
    fn test_decode_raw_measurement() {
        assert_eq!(BarometerDps310::decode_raw_measurement([0x00, 0x00, 0x01]), 1);
        assert_eq!(BarometerDps310::decode_raw_measurement([0x7F, 0xFF, 0xFF]), 8_388_607);
        assert_eq!(BarometerDps310::decode_raw_measurement([0x80, 0x00, 0x00]), -8_388_608);
        assert_eq!(BarometerDps310::decode_raw_measurement([0xFF, 0xFF, 0xFF]), -1);
    }

    #[test]
    fn test_decode_calibration() {
        // c0 = -1, c1 = -2
        // c00 = -3, c10 = -4
        // Remaining coefficients = -1
        let data = [
            0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xDF, 0xFF, 0xFC, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let (temperature, pressure) = BarometerDps310::decode_calibration(data);
        assert_eq!(temperature, TemperatureCalibration { c0: -1, c1: -2 });
        assert_eq!(pressure, PressureCalibration { c00: -3, c10: -4, c01: -1, c11: -1, c20: -1, c21: -1, c30: -1 });
    }

    #[test]
    fn test_oversampling_register_values() {
        assert_eq!(Dps310Oversampling::X1.register_value(), 0);
        assert_eq!(Dps310Oversampling::X2.register_value(), 1);
        assert_eq!(Dps310Oversampling::X4.register_value(), 2);
        assert_eq!(Dps310Oversampling::X8.register_value(), 3);
        assert_eq!(Dps310Oversampling::X16.register_value(), 4);
        assert_eq!(Dps310Oversampling::X32.register_value(), 5);
        assert_eq!(Dps310Oversampling::X64.register_value(), 6);
        assert_eq!(Dps310Oversampling::X128.register_value(), 7);
    }

    #[test]
    fn test_oversampling_scale_factors() {
        assert_eq!(Dps310Oversampling::X1.scale_factor(), 524_288.0);
        assert_eq!(Dps310Oversampling::X2.scale_factor(), 1_572_864.0);
        assert_eq!(Dps310Oversampling::X4.scale_factor(), 3_670_016.0);
        assert_eq!(Dps310Oversampling::X8.scale_factor(), 7_864_320.0);
        assert_eq!(Dps310Oversampling::X16.scale_factor(), 253_952.0);
        assert_eq!(Dps310Oversampling::X32.scale_factor(), 516_096.0);
        assert_eq!(Dps310Oversampling::X64.scale_factor(), 1_040_384.0);
        assert_eq!(Dps310Oversampling::X128.scale_factor(), 2_088_960.0);
    }

    #[test]
    fn test_oversampling_shift() {
        assert!(!Dps310Oversampling::X1.requires_shift());
        assert!(!Dps310Oversampling::X2.requires_shift());
        assert!(!Dps310Oversampling::X4.requires_shift());
        assert!(!Dps310Oversampling::X8.requires_shift());

        assert!(Dps310Oversampling::X16.requires_shift());
        assert!(Dps310Oversampling::X32.requires_shift());
        assert!(Dps310Oversampling::X64.requires_shift());
        assert!(Dps310Oversampling::X128.requires_shift());
    }

    #[test]
    fn test_measurement_duration() {
        let config = Dps310Config::new();

        assert_eq!(config.measurement_time_us(), 13_000);
    }
}
