#![cfg(feature = "madflight_fc3")]
//#![allow(unused)]
#![allow(clippy::similar_names)]

// RPI PICO RP2350
// see <https://madflight.com/Board-FC3/>
// pins: <https://github.com/qqqlab/madflight/blob/main/src/brd/madflight_FC3.h>
// schematic: <https://madflight.com/img/madflight-FC3.pdf>
// For Betaflight configuration files:
// see <https://github.com/betaflight/config/blob/749fff19942fd7b44fa8020a086e1b566054cae9/configs/MADF/MADFLIGHT_FC3/config.h>

use crate::{
    barometer_sensors::Barometer,
    boards::board::{Board, BoardInit, BoardInitError, GpsHardware, ImuContext},
    boards::platform::SharedI2cBus,
    gps::GpsParser,
    magnetometer_sensors::Magnetometer,
    optical_flow_sensors::OpticalFlow,
    rangefinder_sensors::Rangefinder,
};

use imu_sensors::{Imu426xx, ImuAxisOrder, ImuSpiBus};
use motor_mixers::{MotorDriver, MotorDriverQuadDshot, MotorDriverQuadPwm};
use radio_controllers::Radio;

use cyw43_pio::PioSpi;
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

pub fn imu_context(imu: BoardImu) -> ImuContext<BoardImu> {
    ImuContext::new(imu)
}

pub fn board_hardware(init: BoardInit) -> Result<Board<BoardImu>, BoardInitError> {
    static I2C_BUS: StaticCell<SharedI2cBus> = StaticCell::new();
    // NOTE: rp2350 numbers peripheral starting at 0, eg SPI0, SPI0, I2C0, I2C0 etc

    // Take ownership of the raw RP2350 hardware peripherals block
    #[allow(clippy::default_trait_access)]
    let peripherals = embassy_rp::init(Default::default());

    // SPI0
    // #define SPI_1_PINS                  spi_pins_t{.cs=29,.sck=30,.cipo=28,.copi=31,.irq=27}
    let spi0_cs = peripherals.PIN_29;
    let spi0_clk = peripherals.PIN_30;
    let spi0_mosi = peripherals.PIN_31;
    let spi0_miso = peripherals.PIN_28;
    let spi0_tx_dma = peripherals.DMA_CH0;
    let spi0_rx_dma = peripherals.DMA_CH1;
    // Physical pin assigned to capture the gyroscope's INT1 signal wire
    let spi0_interrupt_pin = peripherals.PIN_27;

    // SPI1
    // #define SD_MMC_PINS                 mmc_pins_t{.dat=36,.clk=34,.cmd=35}
    let spi1_clk = peripherals.PIN_34;
    let spi1_mosi = peripherals.PIN_11;
    let spi1_miso = peripherals.PIN_12;
    let spi1_tx_dma = peripherals.DMA_CH2;
    let spi1_rx_dma = peripherals.DMA_CH3;
    let spi1_cs = peripherals.PIN_13;

    // UART0
    // #define UART_0_PINS                 uart_pins_t{.rx=1,.tx=0}
    let uart0_tx = peripherals.PIN_0;
    let uart0_rx = peripherals.PIN_1;
    let uart0_tx_dma = peripherals.DMA_CH4;
    let uart0_rx_dma = peripherals.DMA_CH5;

    // UART1
    // #define UART_1_PINS                 uart_pins_t{.rx=5,.tx=4}
    let uart1_tx = peripherals.PIN_4;
    let uart1_rx = peripherals.PIN_5;
    let uart1_tx_dma = peripherals.DMA_CH6;
    let uart1_rx_dma = peripherals.DMA_CH7;

    // I2C0
    // #define I2C_0_PINS                  i2c_pins_t{.sda=32,.scl=33,.irq=BusI2c::IRQ_NOT_SET} // for barometer, battery, and magnetometer
    let i2c0_scl = peripherals.PIN_33;
    let i2c0_sda = peripherals.PIN_32;

    // I2C1
    // #define I2C_1_PINS                  i2c_pins_t{.sda=2,.scl=3,.irq=BusI2c::IRQ_NOT_SET} // for GPS
    let i2c1_scl = peripherals.PIN_3;
    let i2c1_sda = peripherals.PIN_2;

    // #define MOTOR_PINS                  motor_pins_t{.m0=6,.m1=7,.m2=8,.m3=9} // BR, TR, BL, TL
    let m1 = peripherals.PIN_6;
    let m2 = peripherals.PIN_7;
    let m3 = peripherals.PIN_8;
    let m4 = peripherals.PIN_9;
    let m5 = peripherals.PIN_16;
    let m6 = peripherals.PIN_17;
    let m7 = peripherals.PIN_18;
    let m8 = peripherals.PIN_19;

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

    let uart1 = {
        let mut config = UsartConfig::default();
        config.baudrate = 115_200;
        Uart::new_blocking(peripherals.USART1, uart1_rx, uart1_tx, config)
    };

    // TODO: PIO SPI
    // --- Device 3: PIO0 Backed SPI (Auxiliary Peripheral) ---
    // let aux_pio_spi = Err(AuxiliaryPioInitError::FeatureDisabled);

    let uart0 = {
        let mut uart_config = UartConfig::default();
        uart_config.baudrate = 115_200; // Standard telemetry link velocity [INDEX]
        Uart::new(peripherals.UART0, uart0_tx, uart0_rx, Irqs, uart0_tx_dma, uart0_rx_dma, uart_config)
            .map_err(|_| BoardInitError::UartError)?
    };

    let uart1 = {
        let mut uart_config = UartConfig::default();
        uart_config.baudrate = 115_200;
        Uart::new(peripherals.UART1, uart1_tx, uart1_rx, Irqs, uart1_tx_dma, uart1_rx_dma, uart_config)
            .map_err(|_| BoardInitError::UartError)?
    };

    let i2c0 = {
        let mut i2c_config = I2cConfig::default();
        i2c_config.frequency = 400_000; // Standard Fast-Mode I2C frequency (400 kHz)
        I2c::new_async(peripherals.I2C0, i2c0_scl, i2c0_sda, Irqs, i2c_config)
    };
    let motor_driver_quad_dshot = MotorDriverQuadDshot::new();
    let motor_driver = MotorDriver::QuadDshot(motor_driver_quad_dshot);

    let radio = Radio::new(radio_controllers::RadioType::Mock);

    static I2C_BUS: StaticCell<SharedI2cBus> = StaticCell::new();
    let i2c = I2c::new_async(peripherals.I2C0, i2c0_sda, i2c0_scl, Irqs, config);
    let shared_i2c = I2C_BUS.init(Mutex::new(i2c));

    let barometer = Barometer::new(init.barometer_type, shared_i2c);
    let magnetometer = Magnetometer::new(init.magnetometer_type, shared_i2c);

    /*let gps = match GpsParser::new(init.gps_provider) {
        Some(parser) => {
            Some(GpsHardware {
                ...
            })
        }
        None => None,
    };*/
    gps = None;
    let rangefinder = Rangefinder::new(init.rangefinder_type);
    let optical_flow = OpticalFlow::new(init.optical_flow_type);

    // Map physical device names to logical device names and return.
    Ok(Board { imu, motor_driver, radio, barometer, magnetometer, gps, rangefinder, optical_flow })
}
