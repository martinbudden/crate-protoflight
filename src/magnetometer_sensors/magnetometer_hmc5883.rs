#![cfg(feature = "magnetometer")]

use crate::{
    i2c_bus::SharedI2cBus,
    magnetometer_sensors::{MagnetometerMessage, magnetometer::RxMagnetometer},
};

const _REG_CONFA: u8 = 0x00;
const _REG_CONFB: u8 = 0x01;
const _REG_MODE: u8 = 0x02;
const _REG_DATA: u8 = 0x03;
const _REG_IDA: u8 = 0x0A;

pub struct MagnetometerHmc5883 {
    pub i2c_bus: &'static SharedI2cBus,
}
impl MagnetometerHmc5883 {
    const I2C_ADDRESS: u8 = 0x1E;
    const CHIP_ID: u8 = 0x48;

    pub const fn new(i2c_bus: &'static SharedI2cBus) -> Self {
        Self { i2c_bus }
    }
}

impl RxMagnetometer for MagnetometerHmc5883 {
    fn message(&self) -> MagnetometerMessage {
        MagnetometerMessage::default()
    }
}
