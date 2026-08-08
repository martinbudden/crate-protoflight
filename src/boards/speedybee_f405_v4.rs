//#![cfg(all(feature = "stm32f405", feature = "speedybee_f405_v4"))]
#![cfg(feature = "stm32f405")]
#![allow(unused)]
#![allow(clippy::similar_names)]

// NOTE: stm32 numbers peripheral start at 1, eg SPI1, SPI1, I2C1, I2C2 etc

// See <https://github.com/betaflight/config/blob/master/configs/SPBE/SPEEDYBEEF405V4/config.h> for Betaflight configuration file

use crate::boards::board::{Board, BoardInitError};
use embassy_stm32::{
    bind_interrupts, dma,
    gpio::{Input, Level, Output, Speed},
    mode::Async,
    peripherals,
    spi::{Config as SpiConfig, Spi, mode::Master},
    usart,
    usart::{Config as UartConfig, Uart},
};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;

// Binds the global hardware DMA vectors.
// This creates the type validation struct "Irqs" required by Spi::new.
bind_interrupts!(struct Irqs {
    // Gyro SPI1
    DMA2_STREAM2 => dma::InterruptHandler<peripherals::DMA2_CH2>;
    DMA2_STREAM3 => dma::InterruptHandler<peripherals::DMA2_CH3>;

    // Receiver USART2
    DMA1_STREAM5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
    DMA1_STREAM6 => dma::InterruptHandler<peripherals::DMA1_CH6>;

    // USART2 peripheral interrupt
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

pub fn init() -> Board {
    let peripherals = embassy_stm32::init(Default::default());

    // SPI1
    let spi1_sck = peripherals.PA5;
    let spi1_sdi = peripherals.PA6;
    let spi1_sdo = peripherals.PA7;
    let spi1_tx_dma = peripherals.DMA2_CH2;
    let spi1_rx_dma = peripherals.DMA2_CH3;
    let gyro1_cs = peripherals.PA4;
    let gyro1_exti = peripherals.PC4;

    // SPI2
    let spi2_sck = peripherals.PB13;
    let spi2_sdi = peripherals.PC2;
    let spi2_sdo = peripherals.PC3;
    let max7456_spi_cs = peripherals.PB12;

    // SPI3
    let spi3_sck = peripherals.PB3;
    let spi3_sdi = peripherals.PB4;
    let spi3_sdo = peripherals.PB5;
    let sdcard_spi_cs = peripherals.PC14;

    // UART1
    let uart1_tx = peripherals.PA9;
    let uart1_rx = peripherals.PA10;

    // UART2
    let uart2_tx = peripherals.PA2;
    let uart2_rx = peripherals.PA3;
    let uart2_tx_dma = peripherals.DMA1_CH6;
    let uart2_rx_dma = peripherals.DMA1_CH5;

    let uart3_tx = peripherals.PC10;
    let uart3_rx = peripherals.PC11;

    let uart4_rx = peripherals.PA1;
    let uart4_tx = peripherals.PA0;

    let uart5_rx = peripherals.PD2;

    let uart6_tx = peripherals.PC6;
    let uart6_rx = peripherals.PC7;

    /*
    let spi1 = {
        let mut config = SpiConfig::default();
        config.frequency = embassy_stm32::time::Hertz(10_000_000);
        let spi_bus =
            Spi::new(peripherals.SPI1, spi1_sck, spi1_sdi, spi1_sdo, spi1_tx_dma, spi1_rx_dma, Irqs, config);
        let cs_output = Output::new(gyro1_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, cs_output, Delay)
    };
    */

    let uart2 = {
        let mut config = UartConfig::default();
        config.baudrate = 115_200;
        Uart::new(peripherals.USART2, uart2_rx, uart2_tx, uart2_tx_dma, uart2_rx_dma, Irqs, config)
    };

    // Map physical device names to logical device names and return.
    Board {
        gyro_spi: Err(BoardInitError::GyroNotAvailable),
        sdcard_spi: Err(BoardInitError::SdCardNotAvailable),
        max7456_spi: Err(BoardInitError::Max7456NotAvailable),
        serial_rx_uart: uart2.map_err(|_| BoardInitError::SerialRxUartError),
        msp_uart: Err(BoardInitError::MspUartNotAvailable),
        esc_sensor_uart: Err(BoardInitError::EscSensorUartNotAvailable),
    }
}
