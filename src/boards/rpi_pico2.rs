#![cfg(feature = "rpi_pico2")]
#![allow(unused)]
#![allow(clippy::similar_names)]

use crate::{
    barometer_sensors::Barometer,
    boards::SharedI2cBus,
    boards::board::{Board, BoardInit, BoardInitError, GpsHardware},
    gps::GpsParser,
    magnetometer_sensors::Magnetometer,
    optical_flow_sensors::OpticalFlow,
    rangefinder_sensors::Rangefinder,
};

use imu_sensors::{Imu426xx, ImuAxisOrder, ImuSpiBus};
use motor_mixers::{MotorDriver, MotorDriverQuadDshot, MotorDriverQuadPwm};
use radio_controllers::Radio;

use embassy_rp::{
    Peri, bind_interrupts, dma, gpio,
    gpio::{Input, Level, Output, Pull},
    i2c,
    i2c::{Async as I2cAsync, Config as I2cConfig, I2c},
    peripherals, pio,
    pio::InterruptHandler as PioInterruptHandler,
    spi::{Async as SpiAsync, Config as SpiConfig, Spi},
    uart,
    uart::{Async as UartAsync, Config as UartConfig, Uart},
};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use static_cell::StaticCell;

type BoardSpi =
    ExclusiveDevice<embassy_rp::spi::Spi<'static, peripherals::SPI0, embassy_rp::spi::Async>, Output<'static>, Delay>;

pub type BoardImu = Imu426xx<ImuSpiBus<BoardSpi>>;

#[cfg(feature = "multicore")]
static EXECUTOR_CORE1: embassy_executor::InterruptExecutor = InterruptExecutor::new();
//static EXECUTOR_CORE1: StaticCell<Executor> = StaticCell::new();

// Core 1 needs its own stack space in RAM
#[cfg(feature = "multicore")]
static mut CORE1_STACK: Stack<4096> = Stack::new();

#[cfg(feature = "multicore")]
pub fn start_core1_executor() -> embassy_executor::SendSpawner {}

pub fn board_hardware(init: BoardInit) -> Result<Board<BoardImu>, BoardInitError> {
    static I2C_BUS: StaticCell<SharedI2cBus> = StaticCell::new();
    // NOTE: rp2350 numbers peripheral starting at 0, eg SPI0, SPI0, I2C0, I2C0 etc

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
    let spi1_clk = peripherals.PIN_10;
    let spi1_mosi = peripherals.PIN_11;
    let spi1_miso = peripherals.PIN_12;
    let spi1_tx_dma = peripherals.DMA_CH2;
    let spi1_rx_dma = peripherals.DMA_CH3;
    let spi1_cs = peripherals.PIN_13;

    // UART0
    let uart0_tx = peripherals.PIN_0;
    let uart0_rx = peripherals.PIN_1;
    let uart0_tx_dma = peripherals.DMA_CH4;
    let uart0_rx_dma = peripherals.DMA_CH5;

    // UART1
    let uart1_tx = peripherals.PIN_4;
    let uart1_rx = peripherals.PIN_5;
    let uart1_tx_dma = peripherals.DMA_CH6;
    let uart1_rx_dma = peripherals.DMA_CH7;

    // I2C0
    let i2c0_scl = peripherals.PIN_9;
    let i2c0_sda = peripherals.PIN_8;

    let spi0 = {
        let mut spi_config = SpiConfig::default();
        spi_config.frequency = 10_000_000;
        let spi_bus =
            Spi::new(peripherals.SPI0, spi0_clk, spi0_mosi, spi0_miso, spi0_tx_dma, spi0_rx_dma, Irqs, spi_config);
        let spi_cs_output = Output::new(spi0_cs, Level::High);
        ExclusiveDevice::new(spi_bus, spi_cs_output, embassy_time::Delay).unwrap()
    };
    // Trick to find type of spi
    //let spi1_type: () = spi1;

    let spi0_interrupt = Input::new(spi0_interrupt_pin, embassy_rp::gpio::Pull::Up);
    let mut imu: BoardImu = Imu426xx::new(ImuSpiBus::new(spi0), init.axis_order);

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
        let spi_cs_output = Output::new(spi1_cs, Level::High);
        ExclusiveDevice::new(spi_bus, spi_cs_output, embassy_time::Delay)
    };

    // TODO: PIO SPI
    // --- Device 3: PIO0 Backed SPI (Auxiliary Peripheral) ---
    // let aux_pio_spi = Err(AuxiliaryPioInitError::FeatureDisabled);

    let uart0 = {
        let mut uart_config = UartConfig::default();
        uart_config.baudrate = 115_200; // Standard telemetry link velocity [INDEX]
        Uart::new(peripherals.UART0, uart0_tx, uart0_rx, Irqs, uart0_tx_dma, uart0_rx_dma, uart_config)
    };

    let uart1 = {
        let mut uart_config = UartConfig::default();
        uart_config.baudrate = 115_200;
        Uart::new(peripherals.UART1, uart1_tx, uart1_rx, Irqs, uart1_tx_dma, uart1_rx_dma, uart_config)
    };

    let i2c0 = {
        let mut i2c_config = I2cConfig::default();
        i2c_config.frequency = 400_000; // Standard Fast-Mode I2C frequency (400 kHz)
        //I2c::new_async(peripherals.I2C0, i2c0_scl, i2c0_sda, Irqs, i2c_config)
        I2c::new_blocking(peripherals.I2C0, i2c0_scl, i2c0_sda, i2c_config)
    };
    let motor_driver_quad_dshot = MotorDriverQuadDshot::new();
    let motor_driver = MotorDriver::QuadDshot(motor_driver_quad_dshot);

    let radio = Radio::new(radio_controllers::RadioType::Mock);

    let shared_i2c = I2C_BUS.init(SharedI2cBus::new(i2c0));

    let barometer = Barometer::new(init.barometer_type, shared_i2c);
    let magnetometer = Magnetometer::new(init.magnetometer_type, shared_i2c);
    let gps = None; //GpsParser::new(init.gps_provider);
    let rangefinder = Rangefinder::new(init.rangefinder_type);
    let optical_flow = OpticalFlow::new(init.optical_flow_type);

    // Map physical device names to logical device names and return.
    Ok(Board {
        imu,
        motor_driver,
        radio,

        //sdcard_spi: None,
        // osd_spi: aux_pio_spi,
        //msp_uart: Some(uart1),
        //sensors_i2c: Some(i2c0),
        barometer,
        magnetometer,
        gps,
        rangefinder,
        optical_flow,
        // pub flash: Peri<'static, peripherals::FLASH>,
        // flash: peripherals.FLASH, //
    })
}

// Binds the global hardware DMA vectors.
// This creates the type validation struct "Irqs" required by Spi::new.
bind_interrupts!(pub struct Irqs {
    // Both SPI0 and SPI1 use these DMA channels to handle async wake ups
    DMA_IRQ_0 => dma::InterruptHandler<peripherals::DMA_CH0>,
                 dma::InterruptHandler<peripherals::DMA_CH1>,
                 dma::InterruptHandler<peripherals::DMA_CH2>,
                 dma::InterruptHandler<peripherals::DMA_CH3>,
                 dma::InterruptHandler<peripherals::DMA_CH4>,
                 dma::InterruptHandler<peripherals::DMA_CH5>,
                 dma::InterruptHandler<peripherals::DMA_CH6>,
                 dma::InterruptHandler<peripherals::DMA_CH7>;

    // Used by your 3rd PIO-backed SPI device
    PIO0_IRQ_0 => pio::InterruptHandler<peripherals::PIO0>;
    UART0_IRQ => uart::InterruptHandler<peripherals::UART0>;
    UART1_IRQ => uart::InterruptHandler<peripherals::UART1>;
    I2C0_IRQ => i2c::InterruptHandler<peripherals::I2C0>;
});
