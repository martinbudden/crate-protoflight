#![allow(unused)]
#![cfg(feature = "barometer")]

use embassy_time::{Duration, Instant, Timer};

use crate::{
    barometer_sensors::{BarometerDevice, BarometerI2cError, BarometerMessage},
    i2c_bus::{SharedI2cBus, SharedI2cExt},
};

pub struct BarometerBmp280 {
    pub bus: &'static SharedI2cBus,

    address: u8,

    timestamp: Instant,

    temperature_calibration: TemperatureCalibration,
    temperature_fine: i32,
    temperature_celsius: f32,

    pressure_calibration: PressureCalibration,
    pressure_pascals: f32,

    pressure_at_reference_altitude: f32,

    // Minimum sensor measurement cycle interval, not guaranteed application sampling interval.
    measurement_interval: Duration,
}

struct Reg;

#[allow(unused)]
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
    const _CONTROL: u8 = 0xF4;
    const CONFIG: u8 = 0xF5;
    const PRESSURE_MSB: u8 = 0xF7;
    const _PRESSURE_LSB: u8 = 0xF8;
    const _PRESSURE_XLSB: u8 = 0xF9;
    const _TEMPERATURE_MSB: u8 = 0xFA;
    const _TEMPERATURE_LSB: u8 = 0xFB;
    const _TEMPERATURE_XLSB: u8 = 0xFC;
}

#[derive(Clone, Copy)]
struct StandbySetting {
    register_value: u8,
    duration_ms: f32,
}

impl StandbySetting {
    const ALL: [Self; 8] = [
        Self { register_value: 0b000, duration_ms: 0.5 },
        Self { register_value: 0b001, duration_ms: 62.5 },
        Self { register_value: 0b010, duration_ms: 125.0 },
        Self { register_value: 0b011, duration_ms: 250.0 },
        Self { register_value: 0b100, duration_ms: 500.0 },
        Self { register_value: 0b101, duration_ms: 1000.0 },
        Self { register_value: 0b110, duration_ms: 2000.0 },
        Self { register_value: 0b111, duration_ms: 4000.0 },
    ];
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TemperatureCalibration {
    t1: u16,
    t2: i16,
    t3: i16,
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

#[derive(Clone, Copy, Debug)]
struct RawMeasurements {
    pressure: u32,
    temperature: u32,
}

impl BarometerBmp280 {
    const I2C_ADDRESS: u8 = 0x76;
    const I2C_ADDRESS_ALTERNATIVE: u8 = 0x77;

    const CHIP_ID: u8 = 0x58;

    // Oversampling:
    // Temperature x2 = 010
    // Pressure x16 = 101
    //
    // Mode:
    // Normal = 11
    const CTRL_MEAS_VALUE: u8 = 0b_0101_0111;

    // IIR filter x4 = 010.
    const FILTER_VALUE: u8 = 0b010;

    // Temperature x2 and pressure x16 measurement time:
    // 1.25 + 2.3 * 2 + (2.3 * 16 + 0.575)
    // = 43.225 ms
    const MEASUREMENT_TIME_MS: f32 = 43.225;

    pub const fn new(bus: &'static SharedI2cBus) -> Self {
        Self {
            bus,
            address: Self::I2C_ADDRESS,

            timestamp: Instant::MIN,

            temperature_calibration: TemperatureCalibration { t1: 0, t2: 0, t3: 0 },
            temperature_fine: 0,
            temperature_celsius: 0.0,

            pressure_calibration: PressureCalibration { p1: 0, p2: 0, p3: 0, p4: 0, p5: 0, p6: 0, p7: 0, p8: 0, p9: 0 },
            pressure_pascals: 0.0,

            pressure_at_reference_altitude: 101_325.0,

            measurement_interval: Duration::MIN,
        }
    }

    #[allow(unused)]
    #[must_use]
    pub const fn with_alternative_address(mut self) -> Self {
        self.address = Self::I2C_ADDRESS_ALTERNATIVE;
        self
    }

    #[allow(unused)]
    pub fn measurement_interval(&self) -> Duration {
        self.measurement_interval
    }

    #[allow(unused)]
    pub fn set_reference_pressure(&mut self, pressure_pa: f32) {
        self.pressure_at_reference_altitude = pressure_pa;
    }

    async fn read_register(&self, register: u8) -> Result<u8, BarometerI2cError> {
        self.bus.read_reg(self.address, register).await.map_err(BarometerI2cError::I2c)
    }

    async fn write_register(&self, register: u8, value: u8) -> Result<(), BarometerI2cError> {
        self.bus.write_reg(self.address, register, value).await.map_err(BarometerI2cError::I2c)
    }

    async fn read_registers<const N: usize>(&self, register: u8) -> Result<[u8; N], BarometerI2cError> {
        self.bus.read_regs::<N>(self.address, register).await.map_err(BarometerI2cError::I2c)
    }

    // ---------------------------------------------------------
    // Initialization
    // ---------------------------------------------------------

    pub async fn init(&mut self) -> Result<Duration, BarometerI2cError> {
        let chip_id = self.read_register(Reg::CHIP_ID).await?;

        if chip_id != Self::CHIP_ID {
            return Err(BarometerI2cError::InvalidChipId { expected: Self::CHIP_ID, actual: chip_id });
        }

        self.soft_reset().await?;

        self.read_calibration().await?;

        //self.configure().await?;

        Ok(self.measurement_interval)
    }

    async fn soft_reset(&self) -> Result<(), BarometerI2cError> {
        self.write_register(Reg::SOFT_RESET, 0xB6).await?;

        // Allow the device to reset and reload calibration data.
        Timer::after(Duration::from_millis(2)).await;

        // STATUS.im_update is bit 0.
        // Wait for NVM calibration update to finish.
        for _ in 0..10 {
            let status = self.read_register(Reg::STATUS).await?;

            if status & 0x01 == 0 {
                return Ok(());
            }

            Timer::after(Duration::from_millis(1)).await;
        }

        Err(BarometerI2cError::InvalidData)
    }

    async fn configure(&self, config: &Bmp280Config) -> Result<(), BarometerI2cError> {
        // Put sensor into sleep mode before changing CONFIG.
        const CTRL_MEAS_SLEEP: u8 = 0b00;
        self.write_register(Reg::CTRL_MEAS, CTRL_MEAS_SLEEP).await?;
        // CONFIG register:
        // [7:5] standby duration
        // [4:2] IIR filter
        // [1]   SPI 3-wire enable
        let config_reg = (config.standby_duration.register_value() << 5) | (config.filter.register_value() << 2);
        self.write_register(Reg::CONFIG, config_reg).await?;
        // CTRL_MEAS register:
        // [7:5] temperature oversampling
        // [4:2] pressure oversampling
        // [1:0] operating mode
        let ctrl_meas = (config.temperature_oversampling.register_value() << 5)
            | (config.pressure_oversampling.register_value() << 2)
            | config.mode.register_value();
        self.write_register(Reg::CTRL_MEAS, ctrl_meas).await?;
        Ok(())
    }
    async fn configure2(&self, standby: StandbySetting) -> Result<(), BarometerI2cError> {
        // Put sensor in sleep mode before changing CONFIG.
        self.write_register(Reg::CTRL_MEAS, 0b_0101_0100).await?;
        // CONFIG:
        // standby time = selected setting
        // IIR filter = x4
        // SPI 3-wire = disabled
        let config = (standby.register_value << 5) | (Self::FILTER_VALUE << 2);
        self.write_register(Reg::CONFIG, config).await?;
        // Enable normal mode.
        self.write_register(Reg::CTRL_MEAS, Self::CTRL_MEAS_VALUE).await?;
        Ok(())
    }

    async fn read_calibration(&mut self) -> Result<(), BarometerI2cError> {
        let data = self.read_registers::<24>(Reg::CALIBRATION).await?;

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

        // P1 must be nonzero for pressure compensation.
        if self.pressure_calibration.p1 == 0 {
            return Err(BarometerI2cError::InvalidCalibration);
        }

        Ok(())
    }

    async fn read_raw_measurements(&self) -> Result<RawMeasurements, BarometerI2cError> {
        // Read all pressure and temperature registers in
        // one transaction.
        let data = self.read_registers::<6>(Reg::PRESSURE_MSB).await?;
        let pressure = (u32::from(data[0]) << 12) | (u32::from(data[1]) << 4) | (u32::from(data[2]) >> 4);
        let temperature = (u32::from(data[3]) << 12) | (u32::from(data[4]) << 4) | (u32::from(data[5]) >> 4);

        Ok(RawMeasurements { pressure, temperature })
    }

    fn calculate_temperature(&mut self, adc_t: u32) {
        let adc_t = adc_t.cast_signed();

        let calibration = self.temperature_calibration;

        let t1 = i32::from(calibration.t1);
        let t2 = i32::from(calibration.t2);
        let t3 = i32::from(calibration.t3);

        let var1 = (adc_t >> 3) - (t1 << 1);
        let var2 = (adc_t >> 4) - t1;
        let vt1 = var1 * t2;
        let vt2 = ((var2 * var2) >> 12) * t3;

        self.temperature_fine = (vt1 >> 11) + (vt2 >> 14);
        #[allow(clippy::cast_precision_loss)]
        {
            self.temperature_celsius = ((self.temperature_fine * 5 + 128) >> 8) as f32 / 100.0;
        }
    }

    fn calculate_pressure(&mut self, adc_p: u32) -> Result<(), BarometerI2cError> {
        let calibration = self.pressure_calibration;

        let mut vp1 = i64::from(self.temperature_fine) - 128_000;

        let mut vp2 = vp1 * vp1 * i64::from(calibration.p6);
        vp2 += (vp1 * i64::from(calibration.p5)) << 17;
        vp2 += i64::from(calibration.p4) << 35;
        vp1 = ((vp1 * vp1 * i64::from(calibration.p3)) >> 8) + ((vp1 * i64::from(calibration.p2)) << 12);
        vp1 = (((1_i64 << 47) + vp1) * i64::from(calibration.p1)) >> 33;

        if vp1 == 0 {
            return Err(BarometerI2cError::InvalidData);
        }

        let mut p = 1_048_576_i64 - i64::from(adc_p);
        p = (((p << 31) - vp2) * 3125) / vp1;
        let vp1 = (i64::from(calibration.p9) * (p >> 13) * (p >> 13)) >> 25;
        let vp2 = (i64::from(calibration.p8) * p) >> 19;

        p = ((p + vp1 + vp2) >> 8) + (i64::from(calibration.p7) << 4);
        #[allow(clippy::cast_precision_loss)]
        {
            self.pressure_pascals = p as f32 / 256.0;
        }

        if !self.pressure_pascals.is_finite() || self.pressure_pascals <= 0.0 {
            return Err(BarometerI2cError::InvalidData);
        }

        Ok(())
    }

    // ---------------------------------------------------------
    // Measurement
    // ---------------------------------------------------------

    pub async fn make_reading(&mut self) -> Result<(), BarometerI2cError> {
        let raw = self.read_raw_measurements().await?;

        self.calculate_temperature(raw.temperature);

        self.calculate_pressure(raw.pressure)?;

        self.timestamp = Instant::now();

        Ok(())
    }

    // ---------------------------------------------------------
    // Message
    // ---------------------------------------------------------

    pub fn message(&self) -> BarometerMessage {
        let altitude_m =
            BarometerMessage::calculate_altitude_meters(self.pressure_pascals, self.pressure_at_reference_altitude);

        BarometerMessage {
            timestamp: self.timestamp,
            altitude_m,
            #[allow(clippy::cast_possible_truncation)]
            altitude_m_i32: altitude_m as i32,
            pressure_pascals: self.pressure_pascals,
            temperature_celsius: self.temperature_celsius,
        }
    }
}

// -------------------------------------------------------------
// Common interface
// -------------------------------------------------------------

impl BarometerDevice for BarometerBmp280 {
    async fn init(&mut self) -> Result<Duration, BarometerI2cError> {
        BarometerBmp280::init(self).await
    }

    async fn make_reading(&mut self) -> Result<(), BarometerI2cError> {
        BarometerBmp280::make_reading(self).await
    }

    fn message(&self) -> BarometerMessage {
        BarometerBmp280::message(self)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bmp280Config {
    pub temperature_oversampling: Oversampling,
    pub pressure_oversampling: Oversampling,
    pub standby_duration: StandbyDuration,
    pub filter: IirFilter,
    pub mode: OperatingMode,
}

#[derive(Clone, Copy, Debug)]
pub enum Oversampling {
    Skipped,
    X1,
    X2,
    X4,
    X8,
    X16,
}
impl Oversampling {
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

#[derive(Clone, Copy, Debug)]
pub enum StandbyDuration {
    Ms0_5,
    Ms62_5,
    Ms125,
    Ms250,
    Ms500,
    Ms1000,
    Ms2000,
    Ms4000,
}

impl StandbyDuration {
    pub const fn as_duration(self) -> Duration {
        match self {
            Self::Ms0_5 => Duration::from_micros(500),
            Self::Ms62_5 => Duration::from_micros(62_500),
            Self::Ms125 => Duration::from_millis(125),
            Self::Ms250 => Duration::from_millis(250),
            Self::Ms500 => Duration::from_millis(500),
            Self::Ms1000 => Duration::from_millis(1_000),
            Self::Ms2000 => Duration::from_millis(2_000),
            Self::Ms4000 => Duration::from_millis(4_000),
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

#[derive(Clone, Copy, Debug, Default)]
pub enum IirFilter {
    Off,
    X2,
    #[default]
    X4,
    X8,
    X16,
}
impl IirFilter {
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

#[derive(Clone, Copy, Debug, Default)]
pub enum OperatingMode {
    Sleep,
    Forced,
    #[default]
    Normal,
}
impl OperatingMode {
    const fn register_value(self) -> u8 {
        match self {
            Self::Sleep => 0b00,
            Self::Forced => 0b01,
            Self::Normal => 0b11,
        }
    }
}

impl Default for Bmp280Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Bmp280Config {
    pub const ENVIRONMENTAL: Self = Self {
        temperature_oversampling: Oversampling::X2,
        pressure_oversampling: Oversampling::X16,
        standby_duration: StandbyDuration::Ms125,
        filter: IirFilter::X4,
        mode: OperatingMode::Normal,
    };
    pub const fn new() -> Self {
        Self {
            temperature_oversampling: Oversampling::X2,
            pressure_oversampling: Oversampling::X16,
            standby_duration: StandbyDuration::Ms125,
            filter: IirFilter::X4,
            mode: OperatingMode::Normal,
        }
    }
}

impl Bmp280Config {
    pub fn measurement_duration(self) -> Duration {
        let temperature = self.temperature_oversampling.multiplier();
        let pressure = self.pressure_oversampling.multiplier();
        let duration_us = 1_250 + 2_300 * temperature + 2_300 * pressure + if pressure > 0 { 575 } else { 0 };
        Duration::from_micros(u64::from(duration_us))
    }

    pub fn measurement_interval(self) -> Duration {
        self.measurement_duration() + self.standby_duration.as_duration()
    }
}
