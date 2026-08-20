#![cfg(all(feature = "stm32f405", feature = "airb_omnibus_f4"))]
//#![cfg(feature = "stm32f405")]
#![allow(unused)]
#![allow(clippy::similar_names)]

// For Betaflight configuration files:
// see <https://github.com/betaflight/unified-targets/blob/master/configs/default/AIRB-OMNIBUSF4.config>
// and <https://github.com/betaflight/config/blob/master/configs/AIRB/OMNIBUSF4/config.h>.

// This board has onboard flash and no SD card slot.
// The Omnibus F4 SD has an SD card slot, but no MAX7465 chip.

use crate::{
    barometer_sensors::Barometer,
    boards::board::{Board, BoardInit, BoardInitError},
    gps::GpsParser,
    magnetometer_sensors::Magnetometer,
    optical_flow_sensors::OpticalFlow,
    rangefinder_sensors::Rangefinder,
};

use imu_sensors::{ImuAxisOrder, ImuSpiBus, Mpu6050}; // TODO: this is placeholder, change to Mpu6000 when driver is available
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

pub type BoardImu = Mpu6050<ImuSpiBus<BoardSpi>>;

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

    // SPI3 - MAX7456 and Flash
    let spi3_sck = peripherals.PC10;
    let spi3_sdi = peripherals.PC11;
    let spi3_sdo = peripherals.PC12;
    let max7456_spi_cs = peripherals.PA15;
    let flash_spi_cs = peripherals.PB3;

    // I2C1
    let i2c1_scl = peripherals.PB8;
    let i2c1_sda = peripherals.PB9;

    // UART1
    // UART1_TX PA9
    // UART1_RX PA10
    let uart1_tx = peripherals.PA9;
    let uart1_rx = peripherals.PA10;

    // UART3
    // UART3_TX PB10
    // UART3_RX PB11
    let uart3_tx = peripherals.PB10;
    let uart3_rx = peripherals.PB11;

    let uart3 = {
        let mut config = embassy_stm32::usart::Config::default();
        config.baudrate = 115_200;
        Uart::new_blocking(peripherals.USART3, uart3_rx, uart3_tx, config)
    };

    // UART6
    // UART6_TX PC6
    // UART6_RX PC7
    //let uart6_tx = peripherals.PC6;
    //let uart6_rx = peripherals.PC7;

    let spi1 = {
        let mut config = SpiConfig::default();
        config.frequency = embassy_stm32::time::Hertz(10_000_000);
        let spi_bus = Spi::new(peripherals.SPI1, spi1_sck, spi1_sdo, spi1_sdi, spi1_tx_dma, spi1_rx_dma, Irqs, config);
        let cs_output = Output::new(gyro1_spi_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, cs_output, Delay).unwrap()
    };

    let mut imu: BoardImu = Mpu6050::new(ImuSpiBus::new(spi1), init.axis_order);

    /*timer B14 AF9
    # pin B14: TIM12 CH1 (AF9)
    # pin B15: TIM12 CH2 (AF9)
    # pin C06: TIM8 CH1 (AF3)
    # pin C07: TIM8 CH2 (AF3)
    # pin C08: TIM8 CH3 (AF3)
    # pin C09: TIM8 CH4 (AF3)
    # pin B00: TIM3 CH3 (AF2)
    # pin B01: TIM3 CH4 (AF2)
    # pin A03: TIM2 CH4 (AF1)
    # pin A02: TIM2 CH3 (AF1)
    # pin A01: TIM5 CH2 (AF2)
    # pin A08: TIM1 CH1 (AF1)
    # pin A09: TIM1 CH2 (AF1)
    # pin A10: TIM1 CH3 (AF1)
    */
    let pwm1 = peripherals.PB14; // TIM12 CH1 (AF2)
    let pwm2 = peripherals.PB15; // TIM12 CH2 (AF2)
    let pwm3 = peripherals.PC6; // TIM8 CH1 (AF1)
    let pwm4 = peripherals.PC7; // TIM8 CH2 (AF1)
    let pwm5 = peripherals.PC8; // TIM8 CH3 (AF2)
    let pwm6 = peripherals.PC9; // TIM8 CH4 (AF1)

    let pwm_m1_m2 = SimplePwm::new(
        peripherals.TIM12,
        Some(PwmPin::new(pwm1, PushPull)),
        Some(PwmPin::new(pwm2, PushPull)),
        None,
        None,
        Hertz(400),
        CountingMode::EdgeAlignedUp,
    );
    let pwm_m3_m4_m5_m6 = SimplePwm::new(
        peripherals.TIM8,
        Some(PwmPin::new(pwm3, PushPull)),
        Some(PwmPin::new(pwm4, PushPull)),
        Some(PwmPin::new(pwm5, PushPull)),
        Some(PwmPin::new(pwm6, PushPull)),
        Hertz(400),
        CountingMode::EdgeAlignedUp,
    );

    let motor_driver_quad_pwm = MotorDriverQuadPwm::new(pwm_m3_m4_m5_m6);
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
