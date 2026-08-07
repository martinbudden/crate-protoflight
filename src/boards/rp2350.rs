#![cfg(feature = "rp2350")]
#![allow(unused)]
#![allow(clippy::similar_names)]

pub struct Board {
    pub gyro_spi: Result<GyroSpiDevice, core::convert::Infallible>,
    pub gyro_interrupt: GyroInterruptPin,

    pub sdcard_spi: Result<BlackboxSpiDevice, BlackboxInitError>,
    pub osd_spi: Result<AuxiliaryPioSpiDevice, AuxiliaryPioInitError>,

    pub telemetry_uart: PrimaryUartDevice,
    pub receiver_uart: SecondaryUartDevice,

    pub sensors_i2c: PrimaryI2cDevice,

    pub flash: Peri<'static, FLASH>,
}

#[cfg(feature = "multicore")]
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use cyw43_pio::PioSpi;
use embassy_rp::{
    Peri, bind_interrupts, dma, gpio,
    gpio::{Input, Level, Output, Pull},
    i2c,
    i2c::{Async as I2cAsync, I2c},
    peripherals,
    peripherals::{
        DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, DMA_CH4, DMA_CH5, DMA_CH6, DMA_CH7, FLASH, I2C0, PIO0, SPI0, SPI1, UART0,
        UART1,
    },
    pio,
    pio::InterruptHandler as PioInterruptHandler,
    spi::{Async as SpiAsync, Config as SpiConfig, Spi},
    uart,
    uart::{Async as UartAsync, Uart},
};
use embassy_time::Delay; // Pulled from the cyw43-pio dependency

// Binds the global hardware DMA vectors.
// This creates the type validation struct "Irqs" required by Spi::new.
bind_interrupts!(pub struct Irqs {
    // Both SPI0 and SPI1 use these DMA channels to handle async wake ups
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>,
                 dma::InterruptHandler<DMA_CH1>,
                 dma::InterruptHandler<DMA_CH2>,
                 dma::InterruptHandler<DMA_CH3>,
                 dma::InterruptHandler<DMA_CH4>,
                 dma::InterruptHandler<DMA_CH5>,
                 dma::InterruptHandler<DMA_CH6>,
                 dma::InterruptHandler<DMA_CH7>;

    // Used by your 3rd PIO-backed SPI device
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    UART0_IRQ => uart::InterruptHandler<UART0>;
    UART1_IRQ => uart::InterruptHandler<UART1>;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

//use embedded_hal_async::spi::SpiDevice;
use embedded_hal_bus::spi::ExclusiveDevice;
//use imu_sensors::AccUnits::G;

// --- Device 1: Hardware SPI0 (Gyroscope) ---
// Tied to SPI0 running asynchronously via the DMA system
pub type GyroSpiDevice = ExclusiveDevice<Spi<'static, SPI0, SpiAsync>, Output<'static>, Delay>;
pub type GyroInterruptPin = Input<'static>;

// --- Device 2: Hardware SPI1 (Blackbox SD Card) ---
// Tied to SPI1 running asynchronously via the DMA system
pub type BlackboxSpiDevice = ExclusiveDevice<Spi<'static, SPI1, SpiAsync>, Output<'static>, Delay>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlackboxInitError {
    FeatureDisabled,
}

// TODO: placeholder
// --- Device 3: PIO0 Backed SPI (Auxiliary Peripheral - MAX7456) ---
// Fully concrete representation using State Machine 0 on the PIO0 block
pub type AuxiliaryPioSpiDevice = ExclusiveDevice<PioSpi<'static, PIO0, 0>, Output<'static>, Delay>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuxiliaryPioInitError {
    FeatureDisabled,
}

pub type PrimaryUartDevice = Uart<'static, UartAsync>;
pub type SecondaryUartDevice = Uart<'static, UartAsync>;
pub type PrimaryI2cDevice = I2c<'static, I2C0, I2cAsync>;

// --- 1. RASPBERRY PI RP2350 ARCHITECTURE CONFIGURATION ---

pub fn init() -> Board {
    // Take ownership of the raw RP2350 hardware peripherals block
    #[allow(clippy::default_trait_access)]
    let peripherals = embassy_rp::init(Default::default());

    // SPI0
    let spi0_cs = peripherals.PIN_17;
    let spi0_clk = peripherals.PIN_18;
    let spi0_mosi = peripherals.PIN_19;
    let spi0_miso = peripherals.PIN_16;
    let spi0_tx_dma = peripherals.DMA_CH0;
    let spi0_rx_dma = peripherals.DMA_CH1;
    // Physical pin assigned to capture the gyroscope's INT1 signal wire
    let spi0_interrupt_pin = peripherals.PIN_20;

    // SPI1
    let spi1_cs = peripherals.PIN_13;
    let spi1_clk = peripherals.PIN_10;
    let spi1_mosi = peripherals.PIN_11;
    let spi1_miso = peripherals.PIN_12;
    let spi1_tx_dma = peripherals.DMA_CH2;
    let spi1_rx_dma = peripherals.DMA_CH3;

    // UART0
    let uart0 = peripherals.UART0;
    let uart0_tx = peripherals.PIN_0;
    let uart0_rx = peripherals.PIN_1;
    let uart0_tx_dma = peripherals.DMA_CH4;
    let uart0_rx_dma = peripherals.DMA_CH5;

    // UART1
    let uart1 = peripherals.UART1;
    let uart1_tx = peripherals.PIN_4;
    let uart1_rx = peripherals.PIN_5;
    let uart1_tx_dma = peripherals.DMA_CH6;
    let uart1_rx_dma = peripherals.DMA_CH7;

    // I2C0
    let i2c0 = peripherals.I2C0;
    let i2c0_scl = peripherals.PIN_9;
    let i2c0_sda = peripherals.PIN_8;

    // --- Device 1: Hardware SPI0 (Gyroscope) ---
    let spi0: Result<
        ExclusiveDevice<
            Spi<'_, embassy_rp::peripherals::SPI0, embassy_rp::spi::Async>,
            Output<'_>,
            embassy_time::Delay,
        >,
        core::convert::Infallible,
    > = {
        let mut spi_config = SpiConfig::default();
        spi_config.frequency = 10_000_000;
        let spi_bus =
            Spi::new(peripherals.SPI0, spi0_clk, spi0_mosi, spi0_miso, spi0_tx_dma, spi0_rx_dma, Irqs, spi_config);
        let cs_pin = Output::new(spi0_cs, Level::High);
        ExclusiveDevice::new(spi_bus, cs_pin, embassy_time::Delay)
    };
    let spi0_interrupt = Input::new(spi0_interrupt_pin, embassy_rp::gpio::Pull::Up);

    // --- Device 2: Hardware SPI1 (Blackbox SD Card) ---
    let spi1 = {
        let mut spi_config = SpiConfig::default();
        // When an SD card boots up, it starts in native SD mode.
        // To force it into SPI mode, the driver sends raw command sequences (CMD0, CMD8, ACMD41).
        // During this initial negotiation, cards only accept a clock speed between 100 kHz and 400 kHz.
        // Passing anything higher will cause the card to fail to answer.
        spi_config.frequency = 400_000;
        // TODO: increase SPI frequency to 20_000_000 after initialization.

        let spi_bus =
            Spi::new(peripherals.SPI1, spi1_clk, spi1_mosi, spi1_miso, spi1_tx_dma, spi1_rx_dma, Irqs, spi_config);
        let cs_pin = Output::new(spi1_cs, Level::High);
        // Map the infallible output into an Ok Result variant matching the outer structure
        ExclusiveDevice::new(spi_bus, cs_pin, embassy_time::Delay).map_err(|_| unreachable!())
    };

    // TODO: PIO SPI
    // --- Device 3: PIO0 Backed SPI (Auxiliary Peripheral) ---
    let aux_pio_spi = Err(AuxiliaryPioInitError::FeatureDisabled);

    // --- Device 4: Hardware UART0 (Primary Telemetry Subsystem) ---
    let uart0 = {
        let mut uart_config = embassy_rp::uart::Config::default();
        uart_config.baudrate = 115_200; // Standard telemetry link velocity [INDEX]
        Uart::new(uart0, uart0_tx, uart0_rx, Irqs, uart0_tx_dma, uart0_rx_dma, uart_config)
    };

    // --- Device 5: Hardware UART1 (Secondary Telemetry Subsystem) ---
    let uart1 = {
        let mut uart_config = embassy_rp::uart::Config::default();
        uart_config.baudrate = 115_200;
        Uart::new(uart1, uart1_tx, uart1_rx, Irqs, uart1_tx_dma, uart1_rx_dma, uart_config)
    };

    // --- Device 6: Hardware I2C0 (Sensor Subsystem) ---
    let i2c0 = {
        let mut i2c_config = embassy_rp::i2c::Config::default();
        i2c_config.frequency = 400_000; // Standard Fast-Mode I2C frequency (400 kHz)
        I2c::new_async(i2c0, i2c0_scl, i2c0_sda, Irqs, i2c_config)
    };

    Board {
        gyro_spi: spi0,
        gyro_interrupt: spi0_interrupt,
        sdcard_spi: spi1,
        osd_spi: aux_pio_spi,
        receiver_uart: uart0,
        telemetry_uart: uart1,
        sensors_i2c: i2c0,
        flash: peripherals.FLASH,
    }
}
