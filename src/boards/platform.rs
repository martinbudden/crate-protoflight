#![allow(unused)]

use cfg_if::cfg_if;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

cfg_if! {
if #[cfg(feature = "stm32f405")] {


    // Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking
use embassy_stm32::{
    gpio::{Output,Input},
    i2c::{I2c,mode::Master as I2cMaster},
    mode::{Async, Blocking},
    spi::{Spi, mode::Master as SpiMaster},
    usart::Uart,
};

pub type SpiDeviceAsync = embedded_hal_bus::spi::ExclusiveDevice<Spi<'static, Async, SpiMaster>, Output<'static>, embassy_time::Delay>;
pub type SpiDeviceBlocking = embedded_hal_bus::spi::ExclusiveDevice<Spi<'static, Blocking, SpiMaster>, Output<'static>, embassy_time::Delay>;
pub type I2cDeviceBlocking = I2c<'static, Blocking, I2cMaster>;
pub type I2cDeviceAsync = I2c<'static, Async, I2cMaster>;
pub type UartDevice = Uart<'static, Async>;
//pub type GpsUartRx = UartDevice;
//pub type GpsUartTx = UartDevice;
pub type GpsUartRx = embassy_stm32::usart::UartRx<'static, Async>;
pub type GpsUartTx = embassy_stm32::usart::UartTx<'static, Async>;
//pub type TargetUart = embassy_stm32::usart::Uart<'static, embassy_stm32::mode::Async>;

pub type GpioInputPin = Input<'static>;
pub type GpioOutputPin = Output<'static>;
pub type MotorPins = [GpioOutputPin;8];

pub type MotorOutput = Output<'static>;
pub type SharedI2cBus = Mutex<NoopRawMutex, I2cDeviceBlocking>;


} else if #[cfg(feature = "rp2350")] {


// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking

use embassy_time::Delay;

use embedded_hal_bus::spi::ExclusiveDevice;

use embassy_rp::{
    Peri, bind_interrupts, dma, gpio,
    gpio::{Input, Level, Output, Pull},
    i2c,
    i2c::{Async as I2cAsync, I2c, Blocking as I2cBlocking},
    peripherals, pio,
    pio::InterruptHandler as PioInterruptHandler,
    spi::{Async as SpiAsync, Spi},
    uart,
    uart::{Async as UartAsync, Uart},
};

// --- Device 1: Hardware SPI0 (Gyroscope) ---
// Tied to SPI0 running asynchronously via the DMA system
pub type GyroSpiBus = Spi<'static, peripherals::SPI0, SpiAsync>;
pub type GyroSpiDevice = ExclusiveDevice<GyroSpiBus, Output<'static>, Delay>;
pub type GpioInputPin = Input<'static>;
pub type GpioOutputPin = Output<'static>;
pub type MotorPins = [GpioOutputPin;8];

// --- Device 2: Hardware SPI1 (Blackbox SD Card) ---
// Tied to SPI1 running asynchronously via the DMA system
pub type SdSpiBus = Spi<'static, peripherals::SPI1, SpiAsync>;
pub type SdSpiDevice = ExclusiveDevice<SdSpiBus, Output<'static>, Delay>;

// TODO: placeholder
// --- Device 3: PIO0 Backed SPI (Auxiliary Peripheral - MAX7456) ---
// Fully concrete representation using State Machine 0 on the PIO0 block
//pub type AuxiliaryPioSpiDevice = ExclusiveDevice<PioSpi<'static, peripherals::PIO0, 0>, Output<'static>, Delay>;

//pub type I2cDevice0Async = I2c<'static, peripherals::I2C0, I2cAsync>;
pub type I2cDeviceAsync = I2c<'static, peripherals::I2C0, I2cAsync>;
pub type I2cDeviceBlocking = I2c<'static, peripherals::I2C0, I2cBlocking>;
pub type SharedI2cBus = embassy_sync::mutex::Mutex< NoopRawMutex, I2cDeviceBlocking>;

//pub type UartDevice = Uart<'static, UartAsync>;
/*
pub type UartDevice0 = embassy_rp::uart::UartRx<'static, embassy_rp::peripherals::UART0, embassy_rp::uart::Async>;
pub type UartDevice1 = embassy_rp::uart::UartRx<'static, embassy_rp::peripherals::UART1, embassy_rp::uart::Async>;
pub type GpsUartRx   = embassy_rp::uart::UartRx<'static, embassy_rp::peripherals::UART0, embassy_rp::uart::Async>;
pub type GpsUartTx   = embassy_rp::uart::UartTx<'static, embassy_rp::peripherals::UART0, embassy_rp::uart::Async>;
*/
//pub type RawI2c = embassy_rp::i2c::I2c<'static, embassy_rp::peripherals::I2C0, embassy_rp::i2c::Async>;


} else  {


use crate::boards::mock_uart::MockUart;

pub type GpsUartRx = MockUart;
pub type GpsUartTx = MockUart;
pub type I2cDeviceBlocking = crate::i2c_bus::MockI2c;

}}
