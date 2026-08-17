#![cfg(feature = "rp2350")]
#![allow(unused)]

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking

use embassy_time::Delay;

use embedded_hal_bus::spi::ExclusiveDevice;

use embassy_rp::{
    Peri, bind_interrupts, dma, gpio,
    gpio::{Input, Level, Output, Pull},
    i2c,
    i2c::{Async as I2cAsync, Blocking as I2cBlocking, I2c},
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
pub type MotorPins = [GpioOutputPin; 8];

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
pub type SharedI2cBus = embassy_sync::mutex::Mutex<NoopRawMutex, I2cDeviceBlocking>;

//pub type UartDevice = Uart<'static, UartAsync>;
/*
pub type UartDevice0 = embassy_rp::uart::UartRx<'static, embassy_rp::peripherals::UART0, embassy_rp::uart::Async>;
pub type UartDevice1 = embassy_rp::uart::UartRx<'static, embassy_rp::peripherals::UART1, embassy_rp::uart::Async>;
pub type GpsUartRx   = embassy_rp::uart::UartRx<'static, embassy_rp::peripherals::UART0, embassy_rp::uart::Async>;
pub type GpsUartTx   = embassy_rp::uart::UartTx<'static, embassy_rp::peripherals::UART0, embassy_rp::uart::Async>;
*/
//pub type RawI2c = embassy_rp::i2c::I2c<'static, embassy_rp::peripherals::I2C0, embassy_rp::i2c::Async>;
