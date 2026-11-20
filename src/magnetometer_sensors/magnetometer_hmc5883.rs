#![allow(unused)]
#![cfg(feature = "magnetometer")]

use crate::magnetometer_sensors::magnetometer_device::SharedI2cBus;

use super::{MagnetometerCommon, MagnetometerDevice, MagnetometerI2cError};

use embassy_time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct MagnetometerHmc5883 {
    pub common: MagnetometerCommon,
}

impl MagnetometerDevice for MagnetometerHmc5883 {
    async fn init(&mut self) -> Result<Duration, MagnetometerI2cError> {
        MagnetometerHmc5883::init(self).await
    }

    async fn make_reading(&mut self) -> Result<(Instant, [i16; 3]), MagnetometerI2cError> {
        MagnetometerHmc5883::make_reading(self).await
    }
}

struct Reg;

impl Reg {
    const _CONFA: u8 = 0x00;
    const _CONFB: u8 = 0x01;
    const _MODE: u8 = 0x02;
    const _DATA: u8 = 0x03;
    const _IDA: u8 = 0x0A;
}

impl MagnetometerHmc5883 {
    // The HMC5883 only supports a single I2C address.
    const I2C_ADDRESS: u8 = 0x1E;
    const CHIP_ID: u8 = 0x48;

    pub const fn new(bus: &'static SharedI2cBus) -> Self {
        Self { common: MagnetometerCommon::new(bus, Self::I2C_ADDRESS) }
    }
    pub async fn init(&mut self) -> Result<Duration, MagnetometerI2cError> {
        core::future::ready(()).await;
        /*let chip_id = self.common.read_register(Reg::CHIP_ID).await?;

        if chip_id != Self::CHIP_ID {
            return Err(BarometerI2cError::InvalidChipId { expected: Self::CHIP_ID, actual: chip_id });
        }

        self.soft_reset().await?;
        self.read_calibration().await?;
        self.configure().await?;*/

        Ok(Duration::from_micros(25_000))
    }

    pub async fn make_reading(&mut self) -> Result<(Instant, [i16; 3]), MagnetometerI2cError> {
        core::future::ready(()).await;
        self.common.timestamp = Instant::now();
        Ok((self.common.timestamp, [0; 3]))
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_sized<T: Sized + Unpin>() {}

    #[test]
    fn normal_types() {
        is_sized::<MagnetometerHmc5883>();
    }
}
