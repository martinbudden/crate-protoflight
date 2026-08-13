#![cfg(all(feature = "stm32f405", feature = "speedybee_f405_v4"))]
//#![cfg(feature = "stm32f405")]
#![allow(unused)]
#![allow(clippy::similar_names)]

// For Betaflight configuration files:
// see <https://github.com/betaflight/config/blob/master/configs/SPBE/SPEEDYBEEF405V4/config.h>
// and <https://github.com/betaflight/unified-targets/blob/master/configs/default/SPBE-SPEEDYBEEF405V4.config>.

use crate::boards::board::{Board, BoardInit, BoardInitError, ImuContext};

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

pub fn board_init(init: BoardInit) -> Result<Board<BoardImu>, BoardInitError> {
    // NOTE: stm32 numbers peripheral start at 1, eg SPI1, SPI1, I2C1, I2C2 etc
    /*
    Using Betaflight naming convention. For an STM32 SPI master:
    SDO = MCU → peripheral = MOSI = TX DMA
    SDI = peripheral → MCU = MISO = RX DMA
    */

    let peripherals = embassy_stm32::init(Default::default());

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
    let uart1_tx = peripherals.PA9;
    let uart1_rx = peripherals.PA10;

    // UART2
    let uart2_tx = peripherals.PA2;
    let uart2_rx = peripherals.PA3;
    let uart2_tx_dma = peripherals.DMA1_CH6;
    let uart2_rx_dma = peripherals.DMA1_CH5;

    // UART3
    let uart3_tx = peripherals.PC10;
    let uart3_rx = peripherals.PC11;

    // UART4
    let uart4_tx = peripherals.PA0;
    let uart4_rx = peripherals.PA1;

    // UART5 — ESC sensor (RX only)
    let uart5_rx = peripherals.PD2;
    let uart5_rx_dma = peripherals.DMA1_CH0;

    // UART6
    let uart6_tx = peripherals.PC6;
    let uart6_rx = peripherals.PC7;
    let uart6_tx_dma = peripherals.DMA2_CH6;
    let uart6_rx_dma = peripherals.DMA2_CH1;

    let spi1 = {
        let mut config = SpiConfig::default();
        config.frequency = embassy_stm32::time::Hertz(10_000_000);
        let spi_bus = Spi::new(peripherals.SPI1, spi1_sck, spi1_sdo, spi1_sdi, spi1_tx_dma, spi1_rx_dma, Irqs, config);
        let cs_output = Output::new(gyro1_spi_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, cs_output, Delay).unwrap()
    };

    let mut imu: BoardImu = Imu426xx::new(ImuSpiBus::new(spi1), init.axis_order);

    let spi2 = {
        let mut config = SpiConfig::default();
        // When an SD card boots up, it starts in native SD mode.
        // To force it into SPI mode, the driver sends raw command sequences (CMD0, CMD8, ACMD41).
        // During this initial negotiation, cards only accept a clock speed between 100 kHz and 400 kHz.
        // Passing anything higher will cause the card to fail to answer.
        config.frequency = embassy_stm32::time::Hertz(400_000);
        let spi_bus = Spi::new_blocking(peripherals.SPI2, spi2_sck, spi2_sdo, spi2_sdi, config);
        let cs_output = Output::new(max7456_spi_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, cs_output, Delay)
    };

    let spi3 = {
        let mut config = SpiConfig::default();
        config.frequency = embassy_stm32::time::Hertz(10_000_000);
        let spi_bus = Spi::new(peripherals.SPI3, spi3_sck, spi3_sdo, spi3_sdi, spi3_tx_dma, spi3_rx_dma, Irqs, config);
        let cs_output = Output::new(sdcard_spi_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, cs_output, Delay)
    };

    let uart1 = {
        let mut config = UsartConfig::default();
        config.baudrate = 115_200;
        Uart::new_blocking(peripherals.USART1, uart1_rx, uart1_tx, config)
    };

    let uart2 = {
        let mut config = UsartConfig::default();
        config.baudrate = 115_200;
        Uart::new(peripherals.USART2, uart2_rx, uart2_tx, uart2_tx_dma, uart2_rx_dma, Irqs, config)
    };

    let uart3 = {
        let mut config = embassy_stm32::usart::Config::default();
        config.baudrate = 115_200;
        Uart::new_blocking(peripherals.USART3, uart3_rx, uart3_tx, config)
    };

    let uart4 = {
        let mut config = embassy_stm32::usart::Config::default();
        config.baudrate = 115_200;
        Uart::new_blocking(peripherals.UART4, uart4_rx, uart4_tx, config)
    };

    let uart5 = {
        let mut config = embassy_stm32::usart::Config::default();
        config.baudrate = 115_200;
        UartRx::new(peripherals.UART5, uart5_rx, uart5_rx_dma, Irqs, config)
    };

    let uart6 = {
        let mut config = embassy_stm32::usart::Config::default();
        config.baudrate = 115_200;
        Uart::new_blocking(peripherals.USART6, uart6_rx, uart6_tx, config)
    };

    let i2c1 = I2c::new_blocking(peripherals.I2C1, i2c1_scl, i2c1_sda, embassy_stm32::i2c::Config::default());

    let m1 = peripherals.PB6;
    let m2 = peripherals.PB7;
    let m3 = peripherals.PB0;
    let m4 = peripherals.PB1;
    let m5 = peripherals.PC8;
    let m6 = peripherals.PC9;
    let m7 = peripherals.PB10;
    let m8 = peripherals.PA15;

    let pwm_m1_m2 = SimplePwm::new(
        peripherals.TIM4,
        Some(PwmPin::new(m1, PushPull)),
        Some(PwmPin::new(m2, PushPull)),
        None,
        None,
        Hertz(400),
        CountingMode::EdgeAlignedUp,
    );

    let pwm_m3_m4 = SimplePwm::new(
        peripherals.TIM3,
        None,
        None,
        Some(PwmPin::new(m3, PushPull)),
        Some(PwmPin::new(m4, PushPull)),
        Hertz(400),
        CountingMode::EdgeAlignedUp,
    );

    let motor_driver_quad_pwm = MotorDriverQuadPwm::new2(pwm_m1_m2, pwm_m3_m4);
    //let motor_driver_quad_dshot = MotorDriverQuadDshot::new();
    let motor_driver = MotorDriver::QuadPwm(motor_driver_quad_pwm);

    let radio = Radio::new(radio_controllers::RadioType::Mock);

    // Map physical device names to logical device names and return.
    Ok(Board {
        imu,
        motor_driver,
        //serial_rx_uart: Some(uart2.map_err(|_| BoardInitError::SerialRxUartError)?),
        radio,
        sdcard_spi: Some(spi3.map_err(|_| BoardInitError::SdCardError)?),
        max7456_spi: Some(spi2.map_err(|_| BoardInitError::Max7456NotAvailable)?),
        msp_uart: None,
        esc_sensor_uart: None,
        sensors_i2c: Some(i2c1),
    })
}

/*
            SpeedyBee F405 V4
                    │
    ┌───────────────┼────────────────┐
    │               │                │
    DMA             DMA              DMA
    │               │                │
SPI1 gyro       SPI3 SD          USART2 RX/TX
    │               │                │
DMA2 S2/S3      DMA1 S2/S7       DMA1 S5/S6
    │
    └──── UART5 ESC RX
                DMA1 S0


Blocking peripherals:
    SPI2  → MAX7456
    UART4 → MSP
    I2C1  → gyro/barometer sensors
*/

// Binds the global hardware DMA vectors.
// This creates the type validation struct "Irqs" required by Spi::new.
bind_interrupts!(struct Irqs {
    // -----------------------------------------------------------------------
    // SPI1 — Gyroscope
    // -----------------------------------------------------------------------
    DMA2_STREAM2 => dma::InterruptHandler<peripherals::DMA2_CH2>;
    DMA2_STREAM3 => dma::InterruptHandler<peripherals::DMA2_CH3>;

    // -----------------------------------------------------------------------
    // SPI3 — SD card
    // -----------------------------------------------------------------------
    DMA1_STREAM2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
    DMA1_STREAM7 => dma::InterruptHandler<peripherals::DMA1_CH7>;

    // -----------------------------------------------------------------------
    // USART2 — Receiver
    // -----------------------------------------------------------------------
    DMA1_STREAM5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
    DMA1_STREAM6 => dma::InterruptHandler<peripherals::DMA1_CH6>;
    USART2 => usart::InterruptHandler<peripherals::USART2>;

    // -----------------------------------------------------------------------
    // UART5 — ESC sensor
    // -----------------------------------------------------------------------
    DMA1_STREAM0 => dma::InterruptHandler<peripherals::DMA1_CH0>;
    UART5 => usart::InterruptHandler<peripherals::UART5>;
});
