#![cfg(feature = "stm32")]

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking

use embassy_stm32::{
    i2c::{I2c, mode::Master as I2cMaster},
    mode::{Async, Blocking},
    peripherals::USB_OTG_HS as USB,
    usart::{UartRx, UartTx},
    usb::Driver as UsbDriver,
};

pub type UsbSender = embassy_usb::class::cdc_acm::Sender<'static, UsbDriver<'static, USB>>;
pub type UsbReceiver = embassy_usb::class::cdc_acm::Receiver<'static, UsbDriver<'static, USB>>;
pub type UsbDeviceDriver = embassy_usb::UsbDevice<'static, UsbDriver<'static, USB>>;

pub type I2cDeviceBlocking = I2c<'static, Blocking, I2cMaster>;
pub type SharedI2cBus = Mutex<NoopRawMutex, I2cDeviceBlocking>;

pub type RadioUartRx = &'static mut UartRx<'static, Async>;
pub type RadioUartTx = &'static mut UartTx<'static, Async>;

pub type GpsUartRx = &'static mut UartRx<'static, Async>;
pub type GpsUartTx = &'static mut UartTx<'static, Async>;
