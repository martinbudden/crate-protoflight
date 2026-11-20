#![cfg(any(feature = "rp2040", feature = "rp235xa", feature = "rp235xb"))]

use embassy_rp::{
    peripherals::USB,
    uart::{Async, UartRx, UartTx},
    usb::Driver as UsbDriver,
};

pub type UsbSender = embassy_usb::class::cdc_acm::Sender<'static, UsbDriver<'static, USB>>;
pub type UsbReceiver = embassy_usb::class::cdc_acm::Receiver<'static, UsbDriver<'static, USB>>;
pub type UsbDeviceDriver = embassy_usb::UsbDevice<'static, UsbDriver<'static, USB>>;

//pub type I2cDevice0Async = I2c<'static, peripherals::I2C0, Async>;
//pub type AuxiliaryPioSpiDevice = ExclusiveDevice<PioSpi<'static, peripherals::PIO0, 0>, Output<'static>, Delay>;

pub type BufferedRadioUartRx = &'static mut embassy_rp::uart::BufferedUartRx;
pub type RadioUartRx = &'static mut UartRx<'static, Async>;
pub type RadioUartTx = &'static mut UartTx<'static, Async>;

pub type GpsUartRx = &'static mut UartRx<'static, Async>;
pub type GpsUartTx = &'static mut UartTx<'static, Async>;
