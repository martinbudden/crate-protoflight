#![allow(unused)]
use cfg_if::cfg_if;

pub enum BoardInitError {
    GyroNotAvailable,
    GyroError,
    SdCardNotAvailable,
    SdCardError,
    Max7456NotAvailable,
    Max7456Error,
    SerialRxUartNotAvailable,
    SerialRxUartError,
    MspUartNotAvailable,
    MspUartError,
    EscSensorUartNotAvailable,
    EscSensorUartError,
}

cfg_if! {
if #[cfg(feature = "stm32f405")] {

pub struct Board {
    pub gyro_spi: Result<SpiDevice, BoardInitError>,
    pub sdcard_spi: Result<SpiDevice, BoardInitError>,
    pub max7456_spi: Result<SpiDevice, BoardInitError>,

    pub serial_rx_uart: Result<UartDevice, BoardInitError>,
    pub msp_uart: Result<UartDevice, BoardInitError>,

    pub esc_sensor_uart: Result<UartDevice, BoardInitError>,
    //pub sensors_i2c: Result<I2cDevice,BoardInitError>,
}

// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking
use embassy_stm32::{
    gpio::Output,
    mode::Async,
    spi::{Spi, mode::Master},
    usart::Uart,
};

type SpiBus = Spi<'static, Async, Master>;
pub type SpiDevice = embedded_hal_bus::spi::ExclusiveDevice<SpiBus, Output<'static>, embassy_time::Delay>;
pub type UartDevice = Uart<'static, Async>;

} else if #[cfg(feature = "rp2350")] {

pub struct Board {
    pub gyro_spi: Result<GyroSpiDevice, BoardInitError>,
    pub gyro_interrupt: GyroInterruptPin,
    pub sdcard_spi: Result<SdSpiDevice, BoardInitError>,
    //pub osd_spi: AuxiliaryPioSpiDevice,

    pub serial_rx_uart: Result<UartDevice, BoardInitError>,
    pub msp_uart: Result<UartDevice, BoardInitError>,

    pub sensors_i2c: Result<I2cDevice, BoardInitError>,

    pub flash: Peri<'static, peripherals::FLASH>,
}

// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking

// --- Device 1: Hardware SPI0 (Gyroscope) ---
// Tied to SPI0 running asynchronously via the DMA system
pub type GyroSpiBus = Spi<'static, peripherals::SPI0, SpiAsync>;
pub type GyroSpiDevice = ExclusiveDevice<GyroSpiBus, Output<'static>, Delay>;
pub type GyroInterruptPin = Input<'static>;

// --- Device 2: Hardware SPI1 (Blackbox SD Card) ---
// Tied to SPI1 running asynchronously via the DMA system
pub type SdSpiBus = Spi<'static, peripherals::SPI1, SpiAsync>;
pub type SdSpiDevice = ExclusiveDevice<SdSpiBus, Output<'static>, Delay>;

// TODO: placeholder
// --- Device 3: PIO0 Backed SPI (Auxiliary Peripheral - MAX7456) ---
// Fully concrete representation using State Machine 0 on the PIO0 block
//pub type AuxiliaryPioSpiDevice = ExclusiveDevice<PioSpi<'static, peripherals::PIO0, 0>, Output<'static>, Delay>;

pub type UartDevice = Uart<'static, UartAsync>;
pub type I2cDevice = I2c<'static, peripherals::I2C0, I2cAsync>;

use embassy_time::Delay;

use embedded_hal_bus::spi::ExclusiveDevice;

use embassy_rp::{
    Peri, bind_interrupts, dma, gpio,
    gpio::{Input, Level, Output, Pull},
    i2c,
    i2c::{Async as I2cAsync, I2c},
    peripherals, pio,
    pio::InterruptHandler as PioInterruptHandler,
    spi::{Async as SpiAsync, Spi},
    uart,
    uart::{Async as UartAsync, Uart},
};

}}
