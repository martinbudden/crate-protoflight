#![cfg(feature = "madflight_fc2")]
#![allow(unused)]

// ESP32 S3
// see https://madflight.com/Board-ESP-FC2/
// pins: https://github.com/qqqlab/madflight/blob/main/src/brd/madflight_FC3.h
// schematic: https://madflight.com/img/madflight-ESP-FC2.pdf

use crate::boards::{
    SharedI2cBus,
    board::{BoardHardware, BoardInit, BoardInitError},
};

use dshot_codec::DshotSpeed;
use imu_sensors::{ImuMock, MockImuBus};
// use imu_sensors::{Imu426xx, ImuSpiBus};
use motor_mixers::{MotorDriver, MotorDriverDshot, MotorDriverPwm, MotorProtocol};

use static_cell::StaticCell;

use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;

use esp_hal::{
    gpio::{AnyPin, DriveMode, Output},
    ledc::{
        Ledc, LowSpeed,
        channel::{self, Channel, ChannelIFace},
        timer::{self, LSClockSource, TimerIFace},
    },
    peripherals,
    spi::slave::Spi,
    time::Rate,
};

pub type BoardImu = ImuMock<MockImuBus>;
pub type Board = BoardHardware<BoardImu>;

impl Board {
    #[allow(clippy::too_many_lines, clippy::similar_names, clippy::no_effect_underscore_binding)]
    pub fn new(init: &BoardInit) -> Result<Self, BoardInitError> {
        let imu = ImuMock::new(MockImuBus::new(), init.axis_order);

        static I2C_BUS: StaticCell<SharedI2cBus> = StaticCell::new();
        //static RADIO_UART_TX: StaticCell<UartTx<'static, UartAsync>> = StaticCell::new();
        //static RADIO_UART_RX: StaticCell<UartRx<'static, UartAsync>> = StaticCell::new();
        static LEDC_TIMER: static_cell::StaticCell<timer::Timer<'static, LowSpeed>> = static_cell::StaticCell::new();

        // Take ownership of the hardware peripherals block
        #[allow(clippy::default_trait_access)]
        //let peripherals = esp_hal::init(esp_hal::Config::default());
        let peripherals = esp_hal::init(Default::default());

        // SPI0
        // #define SPI_0_PINS spi_pins_t{.cs=17,.sck=16,.cipo=14,.copi=15,.irq=13}
        // #define SD_MMC_PINS mmc_pins_t{.dat=37,.clk=36,.cmd=35}
        let spi0_clk = peripherals.GPIO16;
        let spi0_mosi = peripherals.GPIO15;
        let spi0_miso = peripherals.GPIO14;
        // Physical pin assigned to capture the gyroscope's INT1 signal wire
        let gyro_cs_pin = peripherals.GPIO17;
        let gyro_exti_pin = peripherals.GPIO13;

        // UART0
        // #define UART_0_PINS uart_pins_t{.rx=7,.tx=21}
        let uart0_tx = peripherals.GPIO21;
        let uart0_rx = peripherals.GPIO7;

        // UART1
        // #define UART_1_PINS uart_pins_t{.rx=6,.tx=9}
        let uart1_tx = peripherals.GPIO9;
        let uart1_rx = peripherals.GPIO6;

        // I2C0
        // #define I2C_0_PINS i2c_pins_t{.sda=11,.scl=10,.irq=BusI2c::IRQ_NOT_SET}
        let i2c0_scl = peripherals.GPIO10;
        let i2c0_sda = peripherals.GPIO11;

        // I2C1
        // #define I2C_1_PINS i2c_pins_t{.sda=33,.scl=39,.irq=BusI2c::IRQ_NOT_SET}
        let _i2c1_scl = peripherals.GPIO39;
        let _i2c1_sda = peripherals.GPIO33;

        // #define MOTOR_PINS  motor_pins_t{.m0=1,.m1=2,.m2=3,.m3=4} // BR, TR, BL, TL
        let m1 = peripherals.GPIO1;
        let m2 = peripherals.GPIO2;
        let m3 = peripherals.GPIO3;
        let m4 = peripherals.GPIO4;

        let ledc = Ledc::new(peripherals.LEDC);

        let frequency_hz = u32::from(init.motor_pwm_rate);

        let timer = LEDC_TIMER.init(ledc.timer::<LowSpeed>(timer::Number::Timer0));
        timer
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty14Bit,
                clock_source: LSClockSource::APBClk,
                frequency: Rate::from_hz(frequency_hz),
            })
            .expect("failed to configure LEDC timer");

        let mut ch0 = ledc.channel(channel::Number::Channel0, m1);
        let mut ch1 = ledc.channel(channel::Number::Channel1, m2);
        let mut ch2 = ledc.channel(channel::Number::Channel2, m3);
        let mut ch3 = ledc.channel(channel::Number::Channel3, m4);

        ch0.configure(channel::config::Config { timer, duty_pct: 0, drive_mode: DriveMode::PushPull })
            .expect("failed to configure LEDC channel 0");
        ch1.configure(channel::config::Config { timer, duty_pct: 0, drive_mode: DriveMode::PushPull })
            .expect("failed to configure LEDC channel 1");
        ch2.configure(channel::config::Config { timer, duty_pct: 0, drive_mode: DriveMode::PushPull })
            .expect("failed to configure LEDC channel 2");
        ch3.configure(channel::config::Config { timer, duty_pct: 0, drive_mode: DriveMode::PushPull })
            .expect("failed to configure LEDC channel 3");

        let frequency_hz = f32::from(init.motor_pwm_rate);
        let motor_driver_pwm = MotorDriverPwm::new(ch0, ch1, ch2, ch3, frequency_hz);
        let motor_driver = MotorDriver::Pwm(motor_driver_pwm);

        let radio_uart_tx = None;
        let radio_uart_rx = None;

        let gps_uart_tx = None;
        let gps_uart_rx = None;

        let barometer = None;
        let magnetometer = None;
        let rangefinder = None;
        let optical_flow = None;

        // Map physical device names to logical device names and return.
        Ok(Self {
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
