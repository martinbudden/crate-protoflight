#![cfg(feature = "barometer")]

use crate::barometer_sensors::{BarometerDevice, BarometerMessage};
const _REG_PSR_B2: u8 = 0x00;
const _REG_PSR_B1: u8 = 0x01;
const _REG_PSR_B0: u8 = 0x02;
const _REG_TMP_B2: u8 = 0x03;
const _REG_TMP_B1: u8 = 0x04;
const _REG_TMP_B0: u8 = 0x05;
const _REG_PRS_CFG: u8 = 0x06;
const _REG_TMP_CFG: u8 = 0x07;
const _REG_MEAS_CFG: u8 = 0x08;
const _REG_CFG_REG: u8 = 0x09;

const _REG_RESET: u8 = 0x0C;
const _REG_ID: u8 = 0x0D;

const _REG_COEF: u8 = 0x10;
const _REG_COEF_SRCE: u8 = 0x28;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarometerDps310 {}

impl Default for BarometerDps310 {
    fn default() -> Self {
        Self::new()
    }
}

impl BarometerDps310 {
    const I2C_ADDRESS: u8 = 0x76;
    const CHIP_ID: u8 = 0x11;
    const MAX_SPI_FREQUENCY_HZ: u32 = 10_000_000;

    pub const fn new() -> Self {
        Self {}
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
}
