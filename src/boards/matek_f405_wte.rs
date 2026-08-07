#![cfg(feature = "stm32f405")]
#![allow(unused)]
#![allow(clippy::similar_names)]

// NOTE: stm32 numbers peripheral starting at 1, eg SPI1, SPI1, I2C1, I2C2 etc

pub struct Board {
    pub gyro_spi: GyroSpiDevice,
    pub sdcard_spi: SdSpiDevice,
    pub receiver_uart: UartDevice,
}

pub enum BoardInitError {
    GyroSpiUnavailable,
    SdCardUnavailable,
    OsdUnavailable,
    ReceiverUnavailable,
    TelemetryUnavailable,
    I2cSensorsUnavailable,
}

use embassy_time::Delay;

use embedded_hal_bus::spi::ExclusiveDevice;

use embassy_stm32::{
    bind_interrupts, dma,
    gpio::{Input, Level, Output, Speed},
    mode::Async,
    peripherals,
    spi::{Config as SpiConfig, Spi, mode::Master},
    usart,
    usart::{Config as UartConfig, Uart},
};
// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking

// --- Device 1: Hardware SPI0 (Gyroscope) ---
// Tied to SPI0 running asynchronously via the DMA system
pub type GyroSpiBus = Spi<'static, Async, Master>;
pub type GyroSpiDevice = ExclusiveDevice<GyroSpiBus, Output<'static>, Delay>;
pub type GyroInterruptPin = Input<'static>;

// --- Device 2: Hardware SPI1 (Blackbox SD Card) ---
// Tied to SPI1 running asynchronously via the DMA system
pub type SdSpiBus = Spi<'static, Async, Master>;
pub type SdSpiDevice = ExclusiveDevice<SdSpiBus, Output<'static>, Delay>;

pub type UartDevice = Uart<'static, Async>;

// Binds the global hardware DMA vectors.
// This creates the type validation struct "Irqs" required by Spi::new.
bind_interrupts!(struct Irqs {
    // Gyro SPI1
    DMA2_STREAM2 => dma::InterruptHandler<peripherals::DMA2_CH2>;
    DMA2_STREAM3 => dma::InterruptHandler<peripherals::DMA2_CH3>;

    // SD SPI2
    DMA1_STREAM3 => dma::InterruptHandler<peripherals::DMA1_CH3>;
    DMA1_STREAM4 => dma::InterruptHandler<peripherals::DMA1_CH4>;

    // Receiver UART USART2
    DMA1_STREAM5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
    DMA1_STREAM6 => dma::InterruptHandler<peripherals::DMA1_CH6>;

    // USART2 peripheral interrupt
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

pub fn init() -> Result<Board, BoardInitError> {
    let peripherals = embassy_stm32::init(Default::default());

    // SPI1
    let spi1_peripheral = peripherals.SPI1;
    let spi1_cs = peripherals.PB0;
    let spi1_sck = peripherals.PA5;
    let spi1_mosi = peripherals.PA7;
    let spi1_miso = peripherals.PA6;
    let spi1_tx_dma = peripherals.DMA2_CH3;
    let spi1_rx_dma = peripherals.DMA2_CH2;

    // SPI2
    let spi2_peripheral = peripherals.SPI2;
    let spi2_cs = peripherals.PB12;
    let spi2_sck = peripherals.PB13;
    let spi2_mosi = peripherals.PB15;
    let spi2_miso = peripherals.PB14;
    let spi2_tx_dma = peripherals.DMA1_CH4;
    let spi2_rx_dma = peripherals.DMA1_CH3;

    // UART1

    // UART2
    let uart2_peripheral = peripherals.USART2;
    let uart2_tx = peripherals.PA2;
    let uart2_rx = peripherals.PA3;
    let uart2_tx_dma = peripherals.DMA1_CH6;
    let uart2_rx_dma = peripherals.DMA1_CH5;

    let spi1 = {
        let mut spi_config = SpiConfig::default();
        spi_config.frequency = embassy_stm32::time::Hertz(10_000_000);
        let spi_bus =
            Spi::new(spi1_peripheral, spi1_sck, spi1_mosi, spi1_miso, spi1_tx_dma, spi1_rx_dma, Irqs, spi_config);
        let spi_cs_output = Output::new(spi1_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, spi_cs_output, Delay)
    };

    let spi2 = {
        let mut spi_config = SpiConfig::default();
        // When an SD card boots up, it starts in native SD mode.
        // To force it into SPI mode, the driver sends raw command sequences (CMD0, CMD8, ACMD41).
        // During this initial negotiation, cards only accept a clock speed between 100 kHz and 400 kHz.
        // Passing anything higher will cause the card to fail to answer.
        spi_config.frequency = embassy_stm32::time::Hertz(400_000);
        // TODO: increase SPI frequency to 20_000_000 after initialization.
        let spi_bus =
            Spi::new(spi2_peripheral, spi2_sck, spi2_mosi, spi2_miso, spi2_tx_dma, spi2_rx_dma, Irqs, spi_config);
        let spi_cs_output = Output::new(spi2_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, spi_cs_output, Delay)
    };

    let uart2 = {
        let mut uart_config = UartConfig::default();
        uart_config.baudrate = 115_200;
        Uart::new(uart2_peripheral, uart2_rx, uart2_tx, uart2_tx_dma, uart2_rx_dma, Irqs, uart_config)
    };

    // Map physical device names to logical device names and return.
    Ok(Board {
        gyro_spi: spi1.map_err(|_| BoardInitError::GyroSpiUnavailable)?,
        sdcard_spi: spi2.map_err(|_| BoardInitError::SdCardUnavailable)?,
        receiver_uart: uart2.map_err(|_| BoardInitError::ReceiverUnavailable)?,
    })
}
