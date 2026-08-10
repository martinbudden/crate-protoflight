#![cfg(all(feature = "stm32f405", feature = "matek_f405_wte"))]
#![allow(unused)]

use crate::boards::{
    ImuContext,
    board::{Board, BoardInitError},
};

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
use imu_sensors::{Imu426xx, ImuAxisOrder, ImuSpiBus};
use motor_mixers::MotorDriverQuadPwm;

type BoardSpi =
    ExclusiveDevice<Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Master>, Output<'static>, Delay>;
pub type BoardImu = Imu426xx<ImuSpiBus<BoardSpi>>;

pub fn imu_context(imu: BoardImu) -> ImuContext<BoardImu> {
    ImuContext::new(imu)
}

pub fn board_init(axis_order: ImuAxisOrder) -> Board<BoardImu> {
    // NOTE: stm32 numbers peripheral starting at 1, eg SPI1, SPI1, I2C1, I2C2 etc

    let peripherals = embassy_stm32::init(Default::default());

    // SPI1
    let spi1_sck = peripherals.PA5;
    let spi1_mosi = peripherals.PA7;
    let spi1_miso = peripherals.PA6;
    let spi1_tx_dma = peripherals.DMA2_CH3;
    let spi1_rx_dma = peripherals.DMA2_CH2;
    let spi1_cs = peripherals.PB0;

    // SPI2
    let spi2_sck = peripherals.PB13;
    let spi2_mosi = peripherals.PB15;
    let spi2_miso = peripherals.PB14;
    let spi2_tx_dma = peripherals.DMA1_CH4;
    let spi2_rx_dma = peripherals.DMA1_CH3;
    let spi2_cs = peripherals.PB12;

    // UART1

    // UART2
    let uart2_tx = peripherals.PA2;
    let uart2_rx = peripherals.PA3;
    let uart2_tx_dma = peripherals.DMA1_CH6;
    let uart2_rx_dma = peripherals.DMA1_CH5;

    let spi1 = {
        let mut config = SpiConfig::default();
        config.frequency = embassy_stm32::time::Hertz(10_000_000);
        let spi_bus =
            Spi::new(peripherals.SPI1, spi1_sck, spi1_mosi, spi1_miso, spi1_tx_dma, spi1_rx_dma, Irqs, config);
        let cs_output = Output::new(spi1_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, cs_output, Delay).unwrap()
    };

    let mut imu: BoardImu = Imu426xx::new(ImuSpiBus::new(spi1), axis_order);

    let spi2 = {
        let mut config = SpiConfig::default();
        // When an SD card boots up, it starts in native SD mode.
        // To force it into SPI mode, the driver sends raw command sequences (CMD0, CMD8, ACMD41).
        // During this initial negotiation, cards only accept a clock speed between 100 kHz and 400 kHz.
        // Passing anything higher will cause the card to fail to answer.
        config.frequency = embassy_stm32::time::Hertz(400_000);
        // TODO: increase SPI frequency to 20_000_000 after initialization.
        let spi_bus =
            Spi::new(peripherals.SPI2, spi2_sck, spi2_mosi, spi2_miso, spi2_tx_dma, spi2_rx_dma, Irqs, config);
        let spi_cs_output = Output::new(spi2_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, spi_cs_output, Delay)
    };

    let uart2 = {
        let mut config = UartConfig::default();
        config.baudrate = 115_200;
        Uart::new(peripherals.USART2, uart2_rx, uart2_tx, uart2_tx_dma, uart2_rx_dma, Irqs, config)
    };

    // Map physical device names to logical device names and return.
    Board {
        imu,
        serial_rx_uart: uart2.map_err(|_| BoardInitError::SerialRxUartError),
        motor_driver: Err(BoardInitError::MotorDriverNotAvailable),
        max7456_spi: None,
        sdcard_spi: None,
        msp_uart: None,
        esc_sensor_uart: None,
        sensors_i2c: None,
    }
}

// Binds the global hardware DMA vectors.
// This creates the type validation struct "Irqs" required by Spi::new.
bind_interrupts!(struct Irqs {
    // Gyro SPI1
    DMA2_STREAM2 => dma::InterruptHandler<peripherals::DMA2_CH2>;
    DMA2_STREAM3 => dma::InterruptHandler<peripherals::DMA2_CH3>;

    // SD SPI2
    DMA1_STREAM3 => dma::InterruptHandler<peripherals::DMA1_CH3>;
    DMA1_STREAM4 => dma::InterruptHandler<peripherals::DMA1_CH4>;

    // Receiver USART2
    DMA1_STREAM5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
    DMA1_STREAM6 => dma::InterruptHandler<peripherals::DMA1_CH6>;

    // USART2 peripheral interrupt
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});
