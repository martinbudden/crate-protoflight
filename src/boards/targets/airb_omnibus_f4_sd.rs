#![cfg(feature = "airb_omnibus_f4_sd")]

// For Betaflight configuration files:
// see <https://github.com/betaflight/unified-targets/blob/master/configs/default/AIRB-OMNIBUSF4SD.config>
// and <https://github.com/betaflight/config/blob/master/configs/AIRB/OMNIBUSF4/config.h>.

// This board has onboard flash and no SD card slot.
// The Omnibus F4 SD has an SD card slot, but no MAX7465 chip.

use crate::boards::{
    SharedI2cBus,
    board::{BoardHardware, BoardInit, BoardInitError},
};

use crate::barometer_sensors::Barometer;
use crate::magnetometer_sensors::Magnetometer;
use crate::optical_flow_sensors::OpticalFlow;
use crate::rangefinder_sensors::Rangefinder;

use dshot_codec::{DshotSpeed, DshotWaveform};
use imu_sensors::{ImuSpiBus, Mpu6050}; // TODO: this is placeholder, change to Mpu6000 when driver is available
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
    usart::{Config as UsartConfig, Uart, UartRx, UartTx},
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
#[allow(non_snake_case)]
#[cfg(feature = "realtime_executor")]
#[interrupt]
unsafe fn TIM6_DAC() {
    unsafe {
        REALTIME_EXECUTOR.on_interrupt();
    }
}
#[cfg(feature = "realtime_executor")]
static REALTIME_EXECUTOR: InterruptExecutor = InterruptExecutor::new();

type BoardSpi = ExclusiveDevice<Spi<'static, ModeAsync, SpiMaster>, Output<'static>, Delay>;

pub type BoardImu = Mpu6050<ImuSpiBus<BoardSpi>>;
pub type Board = BoardHardware<BoardImu>;

// TODO: ensure that the dshot buffer instance in a DMA-safe linker section, ie RAM not CCM
//#[link_section = ".dma"]
static DSHOT_WAVEFORM: StaticCell<DshotWaveform> = StaticCell::new();

impl Board {
    #[cfg(feature = "realtime_executor")]
    pub fn realtime_spawner() -> SendSpawner {
        interrupt::TIM6_DAC.set_priority(Priority::P1);
        REALTIME_EXECUTOR.start(interrupt::TIM6_DAC)
    }

    #[allow(clippy::too_many_lines, clippy::similar_names, clippy::no_effect_underscore_binding, unused)]
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

        let spi2_sck = peripherals.PB13;
        let spi2_sdi = peripherals.PB14;
        let spi2_sdo = peripherals.PB15;

        let spi3_sck = peripherals.PC10;
        let spi3_sdi = peripherals.PC11;
        let spi3_sdo = peripherals.PC12;

        let sdcard_spi_cs = peripherals.PB12;
        let sdcard_detect = peripherals.PB7;
        let osd_cs = peripherals.PA15;
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
            let mut config = UsartConfig::default();
            config.baudrate = 115_200;
            Uart::new_blocking(peripherals.USART3, uart3_rx, uart3_tx, config)
        };

        // UART6
        // UART6_TX PC6
        // UART6_RX PC7
        let uart6_tx = peripherals.PC6;
        let uart6_rx = peripherals.PC7;

        let spi1 = {
            let mut config = SpiConfig::default();
            config.frequency = Hertz(10_000_000);
            let spi_bus =
                Spi::new(peripherals.SPI1, spi1_sck, spi1_sdo, spi1_sdi, spi1_tx_dma, spi1_rx_dma, Irqs, config);
            let cs_output = Output::new(gyro1_spi_cs, Level::High, Speed::VeryHigh);
            ExclusiveDevice::new(spi_bus, cs_output, Delay).unwrap()
        };

        // No DMA on spi3
        let spi3 = {
            let mut config = SpiConfig::default();
            config.frequency = Hertz(10_000_000);
            let spi_bus = Spi::new_blocking(peripherals.SPI3, spi3_sck, spi3_sdo, spi3_sdi, config);
            let cs_output = Output::new(sdcard_spi_cs, Level::High, Speed::VeryHigh);
            ExclusiveDevice::new(spi_bus, cs_output, Delay).unwrap()
        };

        let mut imu: BoardImu = Mpu6050::new(ImuSpiBus::new(spi1), init.axis_order);

        let i2c1 = I2c::new_blocking(peripherals.I2C1, i2c1_scl, i2c1_sda, I2cConfig::default());

        /*timer
        # pin B08: TIM10 CH1 (AF3)
        # pin B09: TIM4 CH4 (AF2)
        # pin C06: TIM8 CH1 (AF3)
        # pin C07: TIM8 CH2 (AF3)
        # pin C08: TIM8 CH3 (AF3)
        # pin C09: TIM8 CH4 (AF3)
        # pin B00: TIM3 CH3 (AF2)
        # pin B01: TIM3 CH4 (AF2)
        # pin A03: TIM2 CH4 (AF1)
        # pin A02: TIM2 CH3 (AF1)
        # pin A01: TIM5 CH2 (AF2)
        # pin B06: TIM4 CH1 (AF2)
        # pin A08: TIM1 CH1 (AF1)
        # pin A09: TIM1 CH2 (AF1)
        # pin A10: TIM1 CH3 (AF1)
            */

        let m1 = peripherals.PB0; // TIM3 CH3 (AF2)
        let m2 = peripherals.PB1; // TIM3 CH4 (AF2)
        let m3 = peripherals.PA3; // ITM2 CH4 (AF1)
        let m4 = peripherals.PA2; // TIM2 CH3 (AF1)
        let m5 = peripherals.PA1; // TIM5 CH2 (AF2)
        let m6 = peripherals.PA8; // TIM1 CH1 (AF1)

        let motor_driver = {
            match init.motor_protocol {
                MotorProtocol::Pwm => {
                    let m1_m2 = SimplePwm::new(
                        peripherals.TIM3,
                        None,
                        None,
                        Some(PwmPin::new(m1, PushPull)),
                        Some(PwmPin::new(m2, PushPull)),
                        Hertz(u32::from(init.motor_pwm_rate)),
                        CountingMode::EdgeAlignedUp,
                    );
                    let m4_m3 = SimplePwm::new(
                        peripherals.TIM5,
                        None,
                        Some(PwmPin::new(m5, PushPull)),
                        Some(PwmPin::new(m4, PushPull)),
                        Some(PwmPin::new(m3, PushPull)),
                        Hertz(u32::from(init.motor_pwm_rate)),
                        CountingMode::EdgeAlignedUp,
                    );
                    let m6 = SimplePwm::new(
                        peripherals.TIM1,
                        Some(PwmPin::new(m6, PushPull)),
                        None,
                        None,
                        None,
                        Hertz(400),
                        CountingMode::EdgeAlignedUp,
                    );

                    let motor_driver_pwm = MotorDriverPwm::new(m1_m2, m4_m3, f32::from(init.motor_pwm_rate));
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
                        m3,
                        m4,
                        m5,
                        m6,
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

        /*let (uart2_tx, uart2_rx) = uart2.split();
        let radio_uart_tx = RADIO_UART_TX.init(uart2_tx);
        let radio_uart_rx = RADIO_UART_RX.init(uart2_rx);*/

        let radio_uart_tx = None;
        let radio_uart_rx = None;

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

// Binds the global hardware DMA vectors.
// This creates the type validation struct "Irqs" required by Spi::new.
bind_interrupts!(struct Irqs {
    // -----------------------------------------------------------------------
    // SPI1 — Gyroscope
    // -----------------------------------------------------------------------
    DMA2_STREAM2 => dma::InterruptHandler<peripherals::DMA2_CH2>;
    DMA2_STREAM3 => dma::InterruptHandler<peripherals::DMA2_CH3>;

    DMA2_STREAM1 => dma::InterruptHandler<peripherals::DMA2_CH1>;
});
