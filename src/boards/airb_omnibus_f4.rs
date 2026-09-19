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
    boards::board::{Board, BoardInit, BoardInitError, GpsHardware},
    gps::GpsParser,
    i2c_bus::SharedI2cBus,
    magnetometer_sensors::Magnetometer,
    optical_flow_sensors::OpticalFlow,
    rangefinder_sensors::Rangefinder,
};
use embassy_executor::InterruptExecutor;

use imu_sensors::{ImuAxisOrder, ImuSpiBus, Mpu6050}; // TODO: this is placeholder, change to Mpu6000 when driver is available
use motor_mixers::{MotorDriver, MotorDriverDshot, MotorDriverPwm};

static REALTIME_EXECUTOR: InterruptExecutor = InterruptExecutor::new();

// SAFETY: TIM6_DAC is exclusively reserved for the RealtimeExecutor.
// This handler is the only caller of `on_interrupt()`, and the executor
// has been started before the interrupt is enabled.
#[allow(non_snake_case)]
#[interrupt]
unsafe fn TIM6_DAC() {
    unsafe {
        REALTIME_EXECUTOR.on_interrupt();
    }
}

pub fn start_realtime_executor() -> embassy_executor::SendSpawner {
    interrupt::TIM6_DAC.set_priority(Priority::P1);
    REALTIME_EXECUTOR.start(interrupt::TIM6_DAC)
}

use embassy_stm32::{
    bind_interrupts, dma,
    gpio::{Input, Level, Output, OutputType::PushPull, Pull, Speed},
    i2c::{self, Config as I2cConfig, I2c},
    interrupt,
    interrupt::{InterruptExt, Priority},
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
use static_cell::StaticCell;

type BoardSpi =
    ExclusiveDevice<Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Master>, Output<'static>, Delay>;

pub type BoardImu = Mpu6050<ImuSpiBus<BoardSpi>>;

pub fn board_hardware(init: BoardInit) -> Result<Board<BoardImu>, BoardInitError> {
    // NOTE: stm32 numbers peripherals starting at 1, eg SPI1, SPI2, I2C1, I2C2 etc
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
    // PC6 used by m3, PC7 used by m4
    // let uart6_tx = peripherals.PC6;
    // let uart6_rx = peripherals.PC7;

    let spi1 = {
        let mut config = SpiConfig::default();
        config.frequency = embassy_stm32::time::Hertz(10_000_000);
        let spi_bus = Spi::new(peripherals.SPI1, spi1_sck, spi1_sdo, spi1_sdi, spi1_tx_dma, spi1_rx_dma, Irqs, config);
        let cs_output = Output::new(gyro1_spi_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, cs_output, Delay).unwrap()
    };

    // No DMA on spi3
    let spi3 = {
        let mut config = SpiConfig::default();
        config.frequency = embassy_stm32::time::Hertz(10_000_000);
        let spi_bus = Spi::new_blocking(peripherals.SPI3, spi3_sck, spi3_sdo, spi3_sdi, config);
        let cs_output = Output::new(flash_spi_cs, Level::High, Speed::VeryHigh);
        ExclusiveDevice::new(spi_bus, cs_output, Delay).unwrap()
    };

    let mut imu: BoardImu = Mpu6050::new(ImuSpiBus::new(spi1), init.axis_order);

    let i2c1 = I2c::new_blocking(peripherals.I2C1, i2c1_scl, i2c1_sda, embassy_stm32::i2c::Config::default());

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
    let m1 = peripherals.PB14; // TIM12 CH1 (AF2)
    let m2 = peripherals.PB15; // TIM12 CH2 (AF2)
    let m3 = peripherals.PC6; // TIM8 CH1 (AF1)
    let m4 = peripherals.PC7; // TIM8 CH2 (AF1)
    let m5 = peripherals.PC8; // TIM8 CH3 (AF2)
    let m6 = peripherals.PC9; // TIM8 CH4 (AF1)

    let m1_m2 = SimplePwm::new(
        peripherals.TIM12,
        Some(PwmPin::new(m1, PushPull)),
        Some(PwmPin::new(m2, PushPull)),
        None,
        None,
        Hertz(400),
        CountingMode::EdgeAlignedUp,
    );
    let m3_m4_m5_m6 = SimplePwm::new(
        peripherals.TIM8,
        Some(PwmPin::new(m3, PushPull)),
        Some(PwmPin::new(m4, PushPull)),
        Some(PwmPin::new(m5, PushPull)),
        Some(PwmPin::new(m6, PushPull)),
        Hertz(400),
        CountingMode::EdgeAlignedUp,
    );

    let motor_driver_pwm = MotorDriverPwm::new(m3_m4_m5_m6);
    let motor_driver = MotorDriver::Pwm(motor_driver_pwm);

    let radio = Radio::new(radio_controllers::RadioType::Mock);

    static I2C_BUS: StaticCell<SharedI2cBus> = StaticCell::new();
    let shared_i2c = I2C_BUS.init(SharedI2cBus::new(i2c1));

    let barometer = Barometer::new(init.barometer_type, shared_i2c);
    let magnetometer = Magnetometer::new(init.magnetometer_type, shared_i2c);
    let gps = None; //GpsParser::new(init.gps_provider);
    let rangefinder = Rangefinder::new(init.rangefinder_type);
    let optical_flow = OpticalFlow::new(init.optical_flow_type);

    // Map physical device names to logical device names and return.
    Ok(Board {
        imu,
        motor_driver,
        //serial_rx_uart: None,
        radio,
        barometer,
        magnetometer,
        gps,
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
