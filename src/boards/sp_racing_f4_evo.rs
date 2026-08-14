#![cfg(all(feature = "stm32f405", feature = "sp_racing_f4_evo"))]
//#![cfg(feature = "stm32f405")]
#![allow(unused)]
#![allow(clippy::similar_names)]

// For Betaflight configuration files:
// see <https://github.com/betaflight/unified-targets/blob/master/configs/default/SPRO-SPRACINGF4EVO.config>,
// and <https://github.com/betaflight/config/blob/master/configs/SPRO/SPRACINGF4EVO/config.h>.

use crate::{
    barometer_sensors::Barometer,
    boards::board::{Board, BoardInit, BoardInitError, ImuContext},
    gps::GpsParser,
    magnetometer_sensors::Magnetometer,
    optical_flow_sensors::OpticalFlow,
    rangefinder_sensors::Rangefinder,
};

use imu_sensors::{Imu426xx, ImuAxisOrder, ImuSpiBus};
use motor_mixers::{MotorDriver, MotorDriverQuadDshot, MotorDriverQuadPwm};

use embassy_stm32::{
    bind_interrupts, dma,
    gpio::{Input, Level, Output, OutputType::PushPull, Pull, Speed},
    i2c::{Config as I2cConfig, I2c},
    mode::Async,
    peripherals,
    spi::{Config as SpiConfig, Spi, mode::Master},
    time::Hertz,
    timer::{
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
    },
    usart::{self, Config as UsartConfig, Uart, UartRx},
};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use radio_controllers::Radio;

type BoardSpi =
    ExclusiveDevice<Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Master>, Output<'static>, Delay>;

pub type BoardImu = Imu426xx<ImuSpiBus<BoardSpi>>;

pub fn imu_context(imu: BoardImu) -> ImuContext<BoardImu> {
    ImuContext::new(imu)
}

pub fn board_hardware(init: BoardInit) -> Result<Board<BoardImu>, BoardInitError> {
    // NOTE: stm32 numbers peripheral start at 1, eg SPI1, SPI1, I2C1, I2C2 etc

    let peripherals = embassy_stm32::init(Default::default());

    /*
    Using Betaflight naming convention. For an STM32 SPI master:
    SDO = MCU → peripheral = MOSI = TX DMA
    SDI = peripheral → MCU = MISO = RX DMA
    */

    // SPI1 - Gyroscope
    let spi1_sck = peripherals.PA5;
    let spi1_sdi = peripherals.PA6;
    let spi1_sdo = peripherals.PA7;
    let spi1_tx_dma = peripherals.DMA2_CH3;
    let spi1_rx_dma = peripherals.DMA2_CH2;
    let gyro1_spi_cs = peripherals.PA4;
    let gyro1_exti = peripherals.PC4;

    // SPI2 - MAX7456
    let spi2_sck = peripherals.PB13;
    let spi2_sdi = peripherals.PC2;
    let spi2_sdo = peripherals.PC3;
    let max7456_spi_cs = peripherals.PB12;

    // SPI3 - SD card
    let spi3_sck = peripherals.PB3;
    let spi3_sdi = peripherals.PB4;
    let spi3_sdo = peripherals.PB5;
    let spi3_tx_dma = peripherals.DMA1_CH7;
    let spi3_rx_dma = peripherals.DMA1_CH2;
    let sdcard_spi_cs = peripherals.PC14;

    // I2C1
    let i2c1_scl = peripherals.PB8;
    let i2c1_sda = peripherals.PB9;

    // UART1
    // SERIAL_TX 1 A09
    // SERIAL_RX 1 A10
    let uart1_tx = peripherals.PA9;
    let uart1_rx = peripherals.PA10;

    // UART2
    // SERIAL_TX 2 A02
    // SERIAL_RX 2 A03
    let uart2_tx = peripherals.PA2;
    let uart2_rx = peripherals.PA3;

    let uart2 = {
        let mut config = embassy_stm32::usart::Config::default();
        config.baudrate = 115_200;
        Uart::new_blocking(peripherals.USART2, uart2_rx, uart2_tx, config)
    };

    // UART3
    // SERIAL_TX 3 B10
    // SERIAL_RX 3 B11
    let uart3_tx = peripherals.PB10;
    let uart3_rx = peripherals.PB11;

    // UART4
    // SERIAL_TX 4 C10
    // SERIAL_RX 4 C11
    let uart4_tx = peripherals.PC10;
    let uart4_rx = peripherals.PC11;

    // UART5
    // SERIAL_TX 5 C12
    // SERIAL_RX 5 D02
    let uart5_tx = peripherals.PC12;
    let uart5_rx = peripherals.PD2;

    let spi1 = {
        let mut config = SpiConfig::default();
        config.frequency = embassy_stm32::time::Hertz(10_000_000);
        let spi_bus = Spi::new(peripherals.SPI1, spi1_sck, spi1_sdo, spi1_sdi, spi1_tx_dma, spi1_rx_dma, Irqs, config);
        let cs_output = Output::new(gyro1_spi_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, cs_output, Delay).unwrap()
    };

    let mut imu: BoardImu = Imu426xx::new(ImuSpiBus::new(spi1), init.axis_order);

    let m1 = peripherals.PC6;
    let m2 = peripherals.PC7;
    let m3 = peripherals.PC9;
    let m4 = peripherals.PC8;
    let m5 = peripherals.PB6;
    let m6 = peripherals.PB7;
    let m7 = peripherals.PB1;
    let m8 = peripherals.PB0;

    let pwm_m1_m2_m3_m4 = SimplePwm::new(
        peripherals.TIM8,
        Some(PwmPin::new(m1, PushPull)),
        Some(PwmPin::new(m2, PushPull)),
        Some(PwmPin::new(m4, PushPull)),
        Some(PwmPin::new(m3, PushPull)),
        Hertz(400),
        CountingMode::EdgeAlignedUp,
    );
    let pwm_m5_m6 = SimplePwm::new(
        peripherals.TIM4,
        Some(PwmPin::new(m5, PushPull)),
        Some(PwmPin::new(m6, PushPull)),
        None,
        None,
        Hertz(400),
        CountingMode::EdgeAlignedUp,
    );

    let pwm_m7_m8 = SimplePwm::new(
        peripherals.TIM3,
        None,
        None,
        Some(PwmPin::new(m8, PushPull)),
        Some(PwmPin::new(m7, PushPull)),
        Hertz(400),
        CountingMode::EdgeAlignedUp,
    );

    let motor_driver_quad_pwm = MotorDriverQuadPwm::new(pwm_m1_m2_m3_m4);
    //let motor_driver_quad_dshot = MotorDriverQuadDshot::new();
    let motor_driver = MotorDriver::QuadPwm(motor_driver_quad_pwm);

    let radio = Radio::new(radio_controllers::RadioType::Mock);

    let barometer = Barometer::new(init.barometer_type);
    let magnetometer = Magnetometer::new(init.magnetometer_type);
    let gps_parser = GpsParser::new(init.gps_provider);
    let rangefinder = Rangefinder::new(init.rangefinder_type);
    let optical_flow = OpticalFlow::new(init.optical_flow_type);

    // Map physical device names to logical device names and return.
    Ok(Board {
        imu,
        motor_driver,
        //serial_rx_uart: None,
        radio,
        sdcard_spi: None,
        max7456_spi: None,
        msp_uart: None,
        esc_sensor_uart: None,
        sensors_i2c: None,
        barometer,
        magnetometer,
        gps_parser,
        rangefinder,
        optical_flow,
    })
}

// Binds the global hardware DMA vectors.
// This creates the type validation struct "Irqs" required by Spi::new.
bind_interrupts!(struct Irqs {
    // -----------------------------------------------------------------------
    // SPI1 — Gyroscope
    // -----------------------------------------------------------------------
    DMA2_STREAM2 => dma::InterruptHandler<peripherals::DMA2_CH2>;
    DMA2_STREAM3 => dma::InterruptHandler<peripherals::DMA2_CH3>;

});
