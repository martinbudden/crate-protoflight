#![cfg(feature = "madflight_fc3")]

// RPI PICO RP2350
// see <https://madflight.com/Board-FC3/>
// pins: <https://github.com/qqqlab/madflight/blob/main/src/brd/madflight_FC3.h>
// schematic: <https://madflight.com/img/madflight-FC3.pdf>
// For Betaflight configuration files:
// see <https://github.com/betaflight/config/blob/749fff19942fd7b44fa8020a086e1b566054cae9/configs/MADF/MADFLIGHT_FC3/config.h>

use crate::boards::{
    SharedI2cBus,
    board::{BoardHardware, BoardInit, BoardInitError},
};

use crate::barometer_sensors::Barometer;
use crate::magnetometer_sensors::Magnetometer;
use crate::optical_flow_sensors::OpticalFlow;
use crate::rangefinder_sensors::Rangefinder;

use dshot_codec::DshotSpeed;
use imu_sensors::{Imu426xx, ImuSpiBus};
use motor_mixers::{MotorDriver, MotorDriverDshot, MotorDriverPwm, MotorProtocol};

use static_cell::StaticCell;

#[allow(unused)]
use cyw43_pio::PioSpi;

use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;

#[allow(unused)]
use embassy_rp::{
    Peri, bind_interrupts, dma, gpio,
    gpio::{Input, Level, Output, Pull},
    i2c,
    i2c::{Async as I2cAsync, Config as I2cConfig, I2c},
    peripherals,
    peripherals::PIO1,
    pio,
    pwm::{Config as PwmConfig, Pwm},
    spi::{Async as SpiAsync, Config as SpiConfig, Spi},
    uart,
    uart::{Async as UartAsync, Config as UartConfig, Uart, UartRx, UartTx},
};

#[cfg(feature = "multicore")]
use {
    core::cell::Cell,
    critical_section::Mutex,
    embassy_executor::{Executor, SendSpawner},
    embassy_rp::{
        multicore::{Stack, spawn_core1},
        peripherals::CORE1,
    },
};

// IMU is on SPI_1
type BoardImuSpi = ExclusiveDevice<embassy_rp::spi::Spi<'static, peripherals::SPI1, SpiAsync>, Output<'static>, Delay>;

pub type BoardImu = Imu426xx<ImuSpiBus<BoardImuSpi>>;
pub type Board = BoardHardware<BoardImu>;

impl Board {
    #[cfg(feature = "multicore")]
    pub fn start_core1_executor(core1: Peri<'static, CORE1>) -> SendSpawner {
        static EXECUTOR_CORE1: StaticCell<Executor> = StaticCell::new();
        static mut CORE1_STACK: Stack<4096> = Stack::new();
        static SPAWNER_SLOT: Mutex<Cell<Option<SendSpawner>>> = Mutex::new(Cell::new(None));

        // Spawn core1. The closure doesn't capture any local variables except the Send-safe `core1`.
        spawn_core1(core1, unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) }, move || {
            // Initialize the executor directly on Core 1
            let executor = EXECUTOR_CORE1.init(Executor::new());

            // Start the executor loop
            executor.run(|spawner| {
                // Convert to a SendSpawner and pass it back through our safe global slot
                critical_section::with(|cs| {
                    SPAWNER_SLOT.borrow(cs).set(Some(spawner.make_send()));
                });
            });
        });

        // Back on Core 0, spin-wait until Core 1 writes the spawner into the slot
        loop {
            if let Some(spawner) = critical_section::with(|cs| SPAWNER_SLOT.borrow(cs).take()) {
                return spawner;
            }
        }
    }

    #[allow(clippy::too_many_lines, clippy::similar_names, clippy::no_effect_underscore_binding)]
    pub fn new(init: &BoardInit) -> Result<Self, BoardInitError> {
        // NOTE: rp2350 numbers peripherals starting at 0, eg SPI0, SPI1, I2C0, I2C1 etc

        static I2C_BUS: StaticCell<SharedI2cBus> = StaticCell::new();
        static RADIO_UART_TX: StaticCell<UartTx<'static, UartAsync>> = StaticCell::new();
        static RADIO_UART_RX: StaticCell<UartRx<'static, UartAsync>> = StaticCell::new();

        // Take ownership of the hardware peripherals block
        #[allow(clippy::default_trait_access)]
        let peripherals = embassy_rp::init(Default::default());

        // SPI0
        // #define SD_MMC_PINS mmc_pins_t{.dat=36,.clk=34,.cmd=35}
        let spi0_clk = peripherals.PIN_34;
        let spi0_mosi = peripherals.PIN_35;
        let spi0_miso = peripherals.PIN_36;
        let spi0_tx_dma = peripherals.DMA_CH2;
        let spi0_rx_dma = peripherals.DMA_CH3;
        let sdcard_cs_pin = peripherals.PIN_13;

        // SPI1
        // #define SPI_1_PINS spi_pins_t{.cs=29,.sck=30,.cipo=28,.copi=31,.irq=27}
        let spi1_clk = peripherals.PIN_30;
        let spi1_mosi = peripherals.PIN_31;
        let spi1_miso = peripherals.PIN_28;
        let spi1_tx_dma = peripherals.DMA_CH0;
        let spi1_rx_dma = peripherals.DMA_CH1;
        // Physical pin assigned to capture the gyroscope's INT1 signal wire
        let gyro_cs_pin = peripherals.PIN_29;
        let gyro_exti_pin = peripherals.PIN_27;
        //let gyro_clkin_pin = peripherals.PIN_26; // needed for for ICM42688P,ICP45686

        // UART0
        // #define UART_0_PINS uart_pins_t{.rx=1,.tx=0}
        let uart0_tx = peripherals.PIN_0;
        let uart0_rx = peripherals.PIN_1;
        let uart0_tx_dma = peripherals.DMA_CH4;
        let uart0_rx_dma = peripherals.DMA_CH5;

        // UART1
        // #define UART_1_PINS uart_pins_t{.rx=5,.tx=4}
        let uart1_tx = peripherals.PIN_4;
        let uart1_rx = peripherals.PIN_5;
        let uart1_tx_dma = peripherals.DMA_CH6;
        let uart1_rx_dma = peripherals.DMA_CH7;

        // I2C0
        // #define I2C_0_PINS i2c_pins_t{.sda=32,.scl=33,.irq=BusI2c::IRQ_NOT_SET} // for barometer, battery, and magnetometer
        let i2c0_scl = peripherals.PIN_33;
        let i2c0_sda = peripherals.PIN_32;

        // I2C1
        // #define I2C_1_PINS i2c_pins_t{.sda=2,.scl=3,.irq=BusI2c::IRQ_NOT_SET} // for GPS
        let _i2c1_scl = peripherals.PIN_3;
        let _i2c1_sda = peripherals.PIN_2;

        // #define MOTOR_PINS motor_pins_t{.m0=6,.m1=7,.m2=8,.m3=9} // BR, TR, BL, TL
        let m1 = peripherals.PIN_6;
        let m2 = peripherals.PIN_7;
        let m3 = peripherals.PIN_8;
        let m4 = peripherals.PIN_9;
        let _m5 = peripherals.PIN_16;
        let _m6 = peripherals.PIN_17;
        let _m7 = peripherals.PIN_18;
        let _m8 = peripherals.PIN_19;

        // NOTE: IMU is on SPI_1
        let spi1 = {
            let mut spi_config = SpiConfig::default();
            spi_config.frequency = 10_000_000;
            let spi_bus =
                Spi::new(peripherals.SPI1, spi1_clk, spi1_mosi, spi1_miso, spi1_tx_dma, spi1_rx_dma, Irqs, spi_config);
            let spi_cs_output = Output::new(gyro_cs_pin, Level::High);
            ExclusiveDevice::new(spi_bus, spi_cs_output, embassy_time::Delay).unwrap()
        };
        // Trick to find type of spi
        //let spi1_type: () = spi1;

        let _spi1_interrupt = Input::new(gyro_exti_pin, embassy_rp::gpio::Pull::Up);
        let imu: BoardImu = Imu426xx::new(ImuSpiBus::new(spi1), init.axis_order);

        let _spi0 = {
            let mut spi_config = SpiConfig::default();
            // When an SD card boots up, it starts in native SD mode.
            // To force it into SPI mode, the driver sends raw command sequences (CMD0, CMD8, ACMD41).
            // During this initial negotiation, cards only accept a clock speed between 100 kHz and 400 kHz.
            // Passing anything higher will cause the card to fail to answer.
            spi_config.frequency = 400_000;
            // TODO: increase SPI frequency to 20_000_000 after initialization.
            let spi_bus =
                Spi::new(peripherals.SPI0, spi0_clk, spi0_mosi, spi0_miso, spi0_tx_dma, spi0_rx_dma, Irqs, spi_config);
            let spi_cs_output = Output::new(sdcard_cs_pin, Level::High);
            ExclusiveDevice::new(spi_bus, spi_cs_output, embassy_time::Delay)
        };

        // TODO: PIO SPI
        // --- Device 3: PIO0 Backed SPI (Auxiliary Peripheral) ---
        // let aux_pio_spi = Err(AuxiliaryPioInitError::FeatureDisabled);
        let _uart0 = {
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

        // TODO: PIO0 UART and SPI
        // TODO: PIO2 Dshot motors 5-8
        let motor_driver = {
            match init.motor_protocol {
                MotorProtocol::Pwm => {
                    let config0 = PwmConfig::default();
                    let config1 = PwmConfig::default();

                    let pwm0 = Pwm::new_output_ab(peripherals.PWM_SLICE3, m1, m2, config0);
                    let pwm1 = Pwm::new_output_ab(peripherals.PWM_SLICE4, m3, m4, config1);

                    let frequency_hz = f32::from(init.motor_pwm_rate);
                    let motor_driver_pwm = MotorDriverPwm::new(pwm0, pwm1, frequency_hz);
                    MotorDriver::Pwm(motor_driver_pwm)
                }
                MotorProtocol::Dshot150 | MotorProtocol::Dshot300 | MotorProtocol::Dshot600 => {
                    let dshot_speed = DshotSpeed::try_from(init.motor_protocol);
                    let Ok(dshot_speed) = dshot_speed else {
                        return Err(BoardInitError::MotorProtocolNotSupported);
                    };
                    let motor_driver_dshot = MotorDriverDshot::new(
                        peripherals.PIO1,
                        Irqs,
                        m1,
                        m2,
                        m3,
                        m4,
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

        let (uart1_tx, uart1_rx) = uart1.split();

        let radio_uart_tx = Some(RADIO_UART_TX.init(uart1_tx));
        let radio_uart_rx = Some(RADIO_UART_RX.init(uart1_rx));

        let gps_uart_tx = None;
        let gps_uart_rx = None;

        let shared_i2c = I2C_BUS.init(SharedI2cBus::new(i2c0));
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
            #[cfg(feature = "multicore")]
            gyro_pid_spawner: Self::start_core1_executor(peripherals.CORE1),
            #[cfg(not(feature = "multicore"))]
            gyro_pid_spawner: init.spawner,
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
    PIO1_IRQ_0 => pio::InterruptHandler<peripherals::PIO1>;
    PIO2_IRQ_0 => pio::InterruptHandler<peripherals::PIO2>;
    UART0_IRQ => uart::InterruptHandler<peripherals::UART0>;
    UART1_IRQ => uart::InterruptHandler<peripherals::UART1>;
    I2C0_IRQ => i2c::InterruptHandler<peripherals::I2C0>;
});
