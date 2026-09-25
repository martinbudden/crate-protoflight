#![cfg(feature = "speedybee_f405_v4")]

// For Betaflight configuration files:
// see <https://github.com/betaflight/config/blob/master/configs/SPBE/SPEEDYBEEF405V4/config.h>
// and <https://github.com/betaflight/unified-targets/blob/master/configs/default/SPBE-SPEEDYBEEF405V4.config>
// and <https://betaflight.com/docs/wiki/boards/current/SPEEDYBEEF405V4>.

use crate::boards::{
    SharedI2cBus,
    board::{BoardHardware, BoardInit, BoardInitError},
};

use crate::barometer_sensors::Barometer;
use crate::magnetometer_sensors::Magnetometer;
use crate::optical_flow_sensors::OpticalFlow;
use crate::rangefinder_sensors::Rangefinder;

use dshot_codec::{DshotSpeed, DshotWaveform};
use imu_sensors::{Imu426xx, ImuSpiBus};
use motor_mixers::{MotorDriver, MotorDriverDshot, MotorDriverPwm, MotorProtocol};

use static_cell::StaticCell;

use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;

use embassy_stm32::{
    Config as Stm32Config, bind_interrupts, dma,
    gpio::{Level, Output, OutputType::PushPull, Speed},
    i2c::{Config as I2cConfig, I2c},
    mode::Async as ModeAsync,
    peripherals,
    spi::{Config as SpiConfig, Spi, mode::Master as SpiMaster},
    time::Hertz,
    timer::{
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
    },
    usart::{self, Config as UsartConfig, Uart, UartRx, UartTx},
};
#[cfg(feature = "realtime_executor")]
use {
    embassy_executor::{InterruptExecutor, SendSpawner},
    embassy_stm32::{
        interrupt,
        interrupt::{InterruptExt, Priority},
    },
};

// SAFETY: TIM6_DAC is exclusively reserved for the RealtimeExecutor.
// This handler is the only caller of `on_interrupt()`, and the executor
// has been started before the interrupt is enabled.
#[cfg(feature = "realtime_executor")]
#[allow(non_snake_case)]
#[interrupt]
unsafe fn TIM6_DAC() {
    unsafe {
        REALTIME_EXECUTOR.on_interrupt();
    }
}
#[cfg(feature = "realtime_executor")]
static REALTIME_EXECUTOR: InterruptExecutor = InterruptExecutor::new();

type BoardSpi = ExclusiveDevice<Spi<'static, ModeAsync, SpiMaster>, Output<'static>, Delay>;

pub type BoardImu = Imu426xx<ImuSpiBus<BoardSpi>>;
pub type Board = BoardHardware<BoardImu>;

// TODO: ensure that the dshot buffer instance in a DMA-safe linker section, ie RAM not CCM
//#[link_section = ".dma"]
static DSHOT_WAVEFORM: StaticCell<DshotWaveform> = StaticCell::new();

/*
The motor pins are supplied as Embassy AnyPins.
The GPIO port is derived from the pins.
The BSRR address comes from Embassy's PAC rather than a magic address.
The motor masks are derived from the actual pins.
The DShot buffer is owned by the driver.
The buffer lives in DMA-accessible SRAM.
StaticCell gives us the 'static buffer without static mut references.
The driver can safely be moved into the Embassy task via the explicit Send implementation.
*/

impl Board {
    #[cfg(feature = "realtime_executor")]
    pub fn realtime_spawner() -> SendSpawner {
        interrupt::TIM6_DAC.set_priority(Priority::P1);
        REALTIME_EXECUTOR.start(interrupt::TIM6_DAC)
    }

    #[allow(clippy::too_many_lines, clippy::similar_names, unused)]
    pub fn new(init: &BoardInit) -> Result<Self, BoardInitError> {
        // NOTE: stm32 numbers peripherals starting at 1, eg SPI1, SPI2, I2C1, I2C2 etc
        /*
        Using Betaflight naming convention. For an STM32 SPI master:
        SDO = MCU → peripheral = MOSI = TX DMA
        SDI = peripheral → MCU = MISO = RX DMA
        */
        static I2C_BUS: StaticCell<SharedI2cBus> = StaticCell::new();
        static RADIO_UART_TX: StaticCell<UartTx<'static, ModeAsync>> = StaticCell::new();
        static RADIO_UART_RX: StaticCell<UartRx<'static, ModeAsync>> = StaticCell::new();

        // Take ownership of the hardware peripherals block
        let peripherals = embassy_stm32::init(Stm32Config::default());

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

        // UART6
        let uart6_tx = peripherals.PC6;
        let uart6_rx = peripherals.PC7;
        //et uart6_tx_dma = peripherals.DMA2_CH6;
        //let uart6_rx_dma = peripherals.DMA2_CH1;

        let spi1 = {
            let mut config = SpiConfig::default();
            config.frequency = Hertz(10_000_000);
            let spi_bus =
                Spi::new(peripherals.SPI1, spi1_sck, spi1_sdo, spi1_sdi, spi1_tx_dma, spi1_rx_dma, Irqs, config);
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
            config.frequency = Hertz(400_000);
            let spi_bus = Spi::new_blocking(peripherals.SPI2, spi2_sck, spi2_sdo, spi2_sdi, config);
            let cs_output = Output::new(max7456_spi_cs, Level::High, Speed::VeryHigh);
            ExclusiveDevice::new(spi_bus, cs_output, Delay)
        };

        let spi3 = {
            let mut config = SpiConfig::default();
            config.frequency = Hertz(10_000_000);
            let spi_bus =
                Spi::new(peripherals.SPI3, spi3_sck, spi3_sdo, spi3_sdi, spi3_tx_dma, spi3_rx_dma, Irqs, config);
            let cs_output = Output::new(sdcard_spi_cs, Level::High, Speed::VeryHigh);
            ExclusiveDevice::new(spi_bus, cs_output, Delay)
        };

        let uart1 = {
            let mut config = UsartConfig::default();
            config.baudrate = 115_200;
            Uart::new_blocking(peripherals.USART1, uart1_rx, uart1_tx, config).map_err(|_| BoardInitError::UartError)?
        };

        let (uart2_tx, uart2_rx) = {
            let mut config = UsartConfig::default();
            config.baudrate = 115_200;
            Uart::new(peripherals.USART2, uart2_rx, uart2_tx, uart2_tx_dma, uart2_rx_dma, Irqs, config)
                //Uart::new_blocking(peripherals.USART2, uart2_rx, uart2_tx, config)
                .map_err(|_| BoardInitError::UartError)?
                .split()
        };

        let uart3 = {
            let mut config = UsartConfig::default();
            config.baudrate = 115_200;
            Uart::new_blocking(peripherals.USART3, uart3_rx, uart3_tx, config).map_err(|_| BoardInitError::UartError)?
        };

        let uart4 = {
            let mut config = UsartConfig::default();
            config.baudrate = 115_200;
            Uart::new_blocking(peripherals.UART4, uart4_rx, uart4_tx, config).map_err(|_| BoardInitError::UartError)?
        };

        let uart5 = {
            let mut config = UsartConfig::default();
            config.baudrate = 115_200;
            UartRx::new_blocking(peripherals.UART5, uart5_rx, config).map_err(|_| BoardInitError::UartError)?
        };

        /*let uart6 = {
            let mut config = UsartConfig::default();
            config.baudrate = 115_200;
            Uart::new(peripherals.USART6, uart6_rx, uart6_tx, uart6_tx_dma, uart6_rx_dma, Irqs, config)
                .map_err(|_| BoardInitError::UartError)?
        };*/
        let uart6 = {
            let mut config = UsartConfig::default();
            config.baudrate = 115_200;
            Uart::new_blocking(peripherals.USART6, uart6_rx, uart6_tx, config).map_err(|_| BoardInitError::UartError)?
        };

        let i2c1 = I2c::new_blocking(peripherals.I2C1, i2c1_scl, i2c1_sda, I2cConfig::default());
        //let i2c1 = I2c::(peripherals.I2C1, i2c1_scl, i2c1_sda, i2c1_tx_dma, i2c1_rx_dma, Irqs, embassy_stm32::i2c::Config::default());

        // Motors
        let m1 = peripherals.PB6;
        let m2 = peripherals.PB7;
        let m3 = peripherals.PB0;
        let m4 = peripherals.PB1;
        let m5 = peripherals.PC8;
        let m6 = peripherals.PC9;
        let m7 = peripherals.PB10;
        let m8 = peripherals.PA15;

        let motor_driver = {
            match init.motor_protocol {
                MotorProtocol::Pwm => {
                    let pwm_m1_m2 = SimplePwm::new(
                        peripherals.TIM4,
                        Some(PwmPin::new(m1, PushPull)),
                        Some(PwmPin::new(m2, PushPull)),
                        None,
                        None,
                        Hertz(u32::from(init.motor_pwm_rate)),
                        CountingMode::EdgeAlignedUp,
                    );

                    let pwm_m3_m4 = SimplePwm::new(
                        peripherals.TIM3,
                        None,
                        None,
                        Some(PwmPin::new(m3, PushPull)),
                        Some(PwmPin::new(m4, PushPull)),
                        Hertz(u32::from(init.motor_pwm_rate)),
                        CountingMode::EdgeAlignedUp,
                    );

                    let motor_driver_pwm = MotorDriverPwm::new(pwm_m1_m2, pwm_m3_m4, f32::from(init.motor_pwm_rate));
                    MotorDriver::Pwm(motor_driver_pwm)
                }
                MotorProtocol::Dshot150 | MotorProtocol::Dshot300 | MotorProtocol::Dshot600 => {
                    // For the F405:
                    //const F405_MOTOR_MASKS: DshotMotorMasks = DshotMotorMasks::new(6, 7, 0, 1);
                    let dshot_speed = DshotSpeed::try_from(init.motor_protocol);
                    let Ok(dshot_speed) = dshot_speed else {
                        return Err(BoardInitError::MotorProtocolNotSupported);
                    };
                    let dshot_waveform = DSHOT_WAVEFORM.init(DshotWaveform::new());

                    let motor_driver_dshot = MotorDriverDshot::new(
                        peripherals.TIM8,
                        peripherals.DMA2_CH1,
                        Irqs,
                        m1,
                        m2,
                        m3,
                        m4,
                        dshot_waveform,
                        dshot_speed,
                        init.motor_pole_count,
                    );
                    MotorDriver::Dshot(motor_driver_dshot)
                }
                _ => {
                    return Err(BoardInitError::MotorProtocolNotSupported);
                }
            }
        };

        let radio_uart_tx = Some(RADIO_UART_TX.init(uart2_tx));
        let radio_uart_rx = Some(RADIO_UART_RX.init(uart2_rx));

        //let (gps_tx, gps_rx) = uart6.split();
        //let gps = None; //Some(GpsHardware { uart_rx: gps_rx, uart_tx: gps_tx });
        let gps_uart_tx = None;
        let gps_uart_rx = None;

        let shared_i2c = I2C_BUS.init(SharedI2cBus::new(i2c1));
        let barometer = if let Some(barometer_type) = init.barometer_type {
            Barometer::new(barometer_type, shared_i2c)
        } else {
            None
        };
        let magnetometer = if let Some(magnetometer_type) = init.magnetometer_type {
            Magnetometer::new(magnetometer_type, shared_i2c)
        } else {
            None
        };
        let rangefinder =
            if let Some(rangefinder_type) = init.rangefinder_type { Rangefinder::new(rangefinder_type) } else { None };
        let optical_flow = if let Some(optical_flow_type) = init.optical_flow_type {
            OpticalFlow::new(optical_flow_type)
        } else {
            None
        };

        // Map physical device names to logical device names and return.
        Ok(Self {
            gyro_pid_spawner: init.spawner,
            #[cfg(feature = "realtime_executor")]
            realtime_spawner: Self::realtime_spawner(),
            #[cfg(not(feature = "realtime_executor"))]
            realtime_spawner: init.spawner,
            background_spawner: init.spawner,
            imu,
            motor_driver,
            radio_uart_rx,
            radio_uart_tx,
            gps_uart_rx,
            gps_uart_tx,
            barometer,
            magnetometer,
            rangefinder,
            optical_flow,
        })
    }
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
    // USART6 — GPS
    // -----------------------------------------------------------------------
    //DMA2_STREAM1 => dma::InterruptHandler<peripherals::DMA2_CH1>;
    //DMA2_STREAM6 => dma::InterruptHandler<peripherals::DMA2_CH6>;
    //USART6 => usart::InterruptHandler<peripherals::USART6>;

    // -----------------------------------------------------------------------
    // Dshot
    // -----------------------------------------------------------------------
    //DMA1_STREAM2 => embassy_stm32::dma::InterruptHandler<peripherals::DMA1_CH2>;
    DMA2_STREAM1 => dma::InterruptHandler<peripherals::DMA2_CH1>;
});
