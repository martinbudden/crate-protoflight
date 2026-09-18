#![cfg(all(feature = "stm32f405", feature = "sp_racing_f4_evo"))]
//#![cfg(feature = "stm32f405")]
#![allow(unused)]
#![allow(clippy::similar_names)]

// For Betaflight configuration files:
// see <https://github.com/betaflight/unified-targets/blob/master/configs/default/SPRO-SPRACINGF4EVO.config>,
// and <https://github.com/betaflight/config/blob/master/configs/SPRO/SPRACINGF4EVO/config.h>.

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

use imu_sensors::{Imu426xx, ImuAxisOrder, ImuSpiBus};
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

pub type BoardImu = Imu426xx<ImuSpiBus<BoardSpi>>;

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
        Uart::new_blocking(peripherals.USART1, uart1_rx, uart1_tx, config).map_err(|_| BoardInitError::UartError)?
    };

    /*let uart2 = {
        let mut config = UsartConfig::default();
        config.baudrate = 115_200;
        Uart::new(peripherals.USART2, uart2_rx, uart2_tx, uart2_tx_dma, uart2_rx_dma, Irqs, config)
            //Uart::new_blocking(peripherals.USART2, uart2_rx, uart2_tx, config)
            .map_err(|_| BoardInitError::UartError)?
    };*/

    let uart3 = {
        let mut config = embassy_stm32::usart::Config::default();
        config.baudrate = 115_200;
        Uart::new_blocking(peripherals.USART3, uart3_rx, uart3_tx, config).map_err(|_| BoardInitError::UartError)?
    };

    let uart4 = {
        let mut config = embassy_stm32::usart::Config::default();
        config.baudrate = 115_200;
        Uart::new_blocking(peripherals.UART4, uart4_rx, uart4_tx, config).map_err(|_| BoardInitError::UartError)?
    };

    let uart5 = {
        let mut config = embassy_stm32::usart::Config::default();
        config.baudrate = 115_200;
        UartRx::new_blocking(peripherals.UART5, uart5_rx, config).map_err(|_| BoardInitError::UartError)?
    };

    let i2c1 = I2c::new_blocking(peripherals.I2C1, i2c1_scl, i2c1_sda, embassy_stm32::i2c::Config::default());
    //let i2c1 = I2c::(peripherals.I2C1, i2c1_scl, i2c1_sda, i2c1_tx_dma, i2c1_rx_dma, Irqs, embassy_stm32::i2c::Config::default());

    let m1 = peripherals.PB6;
    let m2 = peripherals.PB7;
    let m3 = peripherals.PB0;
    let m4 = peripherals.PB1;
    let _m5 = peripherals.PC8;
    let _m6 = peripherals.PC9;
    let _m7 = peripherals.PB10;
    let _m8 = peripherals.PA15;

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
    /*let pwm_m5_m6 = SimplePwm::new(
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
    );*/

    let motor_driver_pwm = MotorDriverPwm::new2(pwm_m1_m2, pwm_m3_m4);
    let motor_driver = MotorDriver::Pwm(motor_driver_pwm);

    let radio = Radio::new(radio_controllers::RadioType::Mock);

    static I2C_BUS: StaticCell<SharedI2cBus> = StaticCell::new();
    let shared_i2c = I2C_BUS.init(SharedI2cBus::new(i2c1));

    let barometer = Barometer::new(init.barometer_type, shared_i2c);
    let magnetometer = Magnetometer::new(init.magnetometer_type, shared_i2c);

    //let (gps_tx, gps_rx) = uart6.split();
    let gps = None; //Some(GpsHardware { uart_rx: gps_rx, uart_tx: gps_tx });
    let rangefinder = Rangefinder::new(init.rangefinder_type);
    let optical_flow = OpticalFlow::new(init.optical_flow_type);

    // Map physical device names to logical device names and return.
    Ok(Board { imu, motor_driver, radio, barometer, magnetometer, gps, rangefinder, optical_flow })
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
