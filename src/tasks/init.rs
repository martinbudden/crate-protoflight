use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

#[cfg(all(feature = "rp2350", feature = "multicore"))]
use embassy_rp::multicore::{Stack, spawn_core1};
#[cfg(feature = "rp2350")]
use embassy_rp::{
    bind_interrupts, dma,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3},
    spi::{Config as SpiConfig, Spi},
};

// Binds the global hardware DMA vectors.
// This creates the type validation struct "Irqs" required by Spi::new.
#[cfg(feature = "rp2350")]
bind_interrupts!(pub struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>,
                 dma::InterruptHandler<DMA_CH1>,
                 dma::InterruptHandler<DMA_CH2>,
                 dma::InterruptHandler<DMA_CH3>;
});
//#[cfg(feature = "rp2350")]
//use embedded_hal_async::spi::SpiDevice;
#[cfg(feature = "rp2350")]
use embedded_hal_bus::spi::ExclusiveDevice;

use static_cell::StaticCell;

use imu_sensors::{ImuAxesOrder, ImuMock, MockImuBus};
use motor_mixers::{MotorMixerCommon, MotorMixerQuadXPwm};
use radio_controllers::{Rates, RcModes};
use sensor_fusion::MadgwickFilterf32;

use crate::{
    config::{GLOBAL_CONFIG, config_publisher, config_subscriber, fast_config_publisher, fast_config_subscriber},
    flight::{FlightControlMessage, FlightController, ImuFilterBank, RcAdjustments},
    tasks::{
        flight_control_task::{
            FlightControlContext, flight_control_receiver, flight_control_sender, flight_control_task,
        },
        gyro_pid_task::{
            GyroPidContext, gyro_pid_receiver, gyro_pid_sender, gyro_pid_task, setpoint_receiver, setpoint_sender,
        },
        imu_task::{ImuContext, imu_task},
        motor_mixer_task::{MotorMixerContext, motor_mixer_task},
    },
};

#[cfg(feature = "serde")]
use crate::tasks::non_volatile_storage::load_global_configs;

#[cfg(feature = "autopilot")]
use crate::{
    autopilot::pilot::Autopilot,
    tasks::autopilot_task::{AutopilotContext, autopilot_receiver, autopilot_sender, autopilot_task},
};

#[cfg(feature = "barometer")]
use crate::tasks::barometer_task::{BarometerContext, barometer_publisher, barometer_subscriber, barometer_task};

#[cfg(feature = "battery")]
use crate::tasks::battery_task::{BatteryContext, battery_publisher, battery_subscriber, battery_task};

#[cfg(all(feature = "rp2350", feature = "blackbox"))]
use crate::tasks::sd_writer_task::{SdWriterContext, sd_writer_task};
#[cfg(feature = "blackbox")]
use {
    crate::{
        drivers::sd_card::MockSdCard,
        tasks::blackbox_task::{BlackboxContext, blackbox_task},
    },
    blackbox_logger::{Blackbox, FieldSelect},
};

#[cfg(feature = "gps")]
use crate::{
    gps::Geodetic,
    tasks::gps_task::{GpsContext, gps_publisher, gps_subscriber, gps_task},
};

#[cfg(feature = "msp")]
use crate::{
    multiwii_serial_protocol::Msp,
    tasks::msp_task::{MSP_READ_BUF_SIZE, MSP_WRITE_BUF_SIZE, MspContext, msp_task},
};

#[cfg(feature = "optical_flow")]
use crate::tasks::optical_flow_task::{
    OpticalFlowContext, optical_flow_publisher, optical_flow_subscriber, optical_flow_task,
};

#[cfg(feature = "osd")]
use crate::{
    osd::Osd,
    tasks::osd_task::{OsdContext, osd_task},
};

#[cfg(feature = "rangefinder")]
use crate::tasks::rangefinder_task::{
    RangefinderContext, rangefinder_publisher, rangefinder_subscriber, rangefinder_task,
};

#[cfg(feature = "max7456")]
use {crate::display::DisplayPortMax7456, embedded_hal_async::spi::SpiBus};

#[cfg(not(feature = "max7456"))]
use crate::display::DisplayPortMock;

// --- 1. RASPBERRY PI RP2350 ARCHITECTURE CONFIGURATION ---
#[cfg(all(feature = "max7456", feature = "rp2350"))]
pub type ConcreteSpi = embassy_rp::spi::Spi<'static, embassy_rp::peripherals::SPI0, embassy_rp::spi::Async>;

#[cfg(all(feature = "max7456", feature = "rp2350"))]
pub type SharedDisplay = Mutex<CriticalSectionRawMutex, DisplayPortMax7456<&'static mut ConcreteSpi>>;

// --- 2. HOST ARCHITECTURE TESTING / MOCK CONFIGURATION ---
#[cfg(not(feature = "max7456"))]
pub type DisplayPortMutex = Mutex<CriticalSectionRawMutex, DisplayPortMock>;

// Core 1 needs its own stack space in RAM
#[cfg(feature = "multicore")]
static mut CORE1_STACK: Stack<4096> = Stack::new();

/// Create all the contexts for the tasks, and then spawn the tasks.
#[allow(clippy::expect_used)]
#[allow(clippy::too_many_lines)]
pub async fn init(spawner: Spawner) {
    // ****
    // Statically allocate the task contexts.
    // ****
    static IMU_CTX: StaticCell<ImuContext> = StaticCell::new();
    static GYRO_PID_CTX: StaticCell<GyroPidContext> = StaticCell::new();
    static FLIGHT_CONTROL_CTX: StaticCell<FlightControlContext> = StaticCell::new();
    static MOTOR_MIXER_CTX: StaticCell<MotorMixerContext> = StaticCell::new();

    #[cfg(feature = "autopilot")]
    static AUTOPILOT_CTX: StaticCell<AutopilotContext> = StaticCell::new();
    #[cfg(feature = "barometer")]
    static BAROMETER_CTX: StaticCell<BarometerContext> = StaticCell::new();
    #[cfg(feature = "battery")]
    static BATTERY_CTX: StaticCell<BatteryContext> = StaticCell::new();
    #[cfg(feature = "blackbox")]
    static BLACKBOX_CTX: StaticCell<BlackboxContext> = StaticCell::new();
    #[cfg(all(feature = "blackbox", feature = "rp2350"))]
    static SD_WRITER_CTX: StaticCell<SdWriterContext> = StaticCell::new();
    #[cfg(feature = "gps")]
    static GPS_CTX: StaticCell<GpsContext> = StaticCell::new();
    #[cfg(feature = "msp")]
    static MSP_CTX: StaticCell<MspContext> = StaticCell::new();
    #[cfg(feature = "optical_flow")]
    static OPTICAL_FLOW_CTX: StaticCell<OpticalFlowContext> = StaticCell::new();
    #[cfg(feature = "osd")]
    static OSD_CTX: StaticCell<OsdContext> = StaticCell::new();
    #[cfg(feature = "rangefinder")]
    static RANGEFINDER_CTX: StaticCell<RangefinderContext> = StaticCell::new();

    #[cfg(all(feature = "max7456", feature = "rp2350"))]
    static SPI_BUS_CELL: StaticCell<ConcreteSpiType> = StaticCell::new();

    static DISPLAY_PORT_MUTEX_CELL: StaticCell<DisplayPortMutex> = StaticCell::new();

    // Initialize env_logger for logging to stdout on desktop platforms.
    // This connects the logger to the terminal and polls the environment variables.
    #[cfg(feature = "std")]
    env_logger::init();

    // Take ownership of the raw RP2350 hardware peripherals block
    #[cfg(feature = "rp2350")]
    let peripherals = embassy_rp::init(Default::default());

    #[cfg(feature = "rp2350")]
    let _gyro_spi = {
        let mut spi_config = SpiConfig::default();
        spi_config.frequency = 10_000_000;

        // Notice: Irqs is completely omitted from the parameters here.
        let spi_bus = Spi::new(
            peripherals.SPI0,
            peripherals.PIN_18,  // CLK defined internally
            peripherals.PIN_19,  // MOSI defined internally
            peripherals.PIN_16,  // MISO defined internally
            peripherals.DMA_CH0, // TX DMA
            peripherals.DMA_CH1, // RX DMA
            Irqs,
            spi_config,
        );
        let cs_pin = Output::new(unsafe { core::ptr::read(&peripherals.PIN_17) }, Level::High);
        ExclusiveDevice::new(spi_bus, cs_pin, embassy_time::Delay)
    };
    #[cfg(feature = "rp2350")]
    let _blackbox_spi = {
        let mut spi_config = SpiConfig::default();
        spi_config.frequency = 400_000;

        // Notice: Irqs is completely omitted from the parameters here.
        let spi_bus = Spi::new(
            peripherals.SPI1,
            peripherals.PIN_10,  // CLK defined internally
            peripherals.PIN_11,  // MOSI defined internally
            peripherals.PIN_12,  // MISO defined internally
            peripherals.DMA_CH2, // TX DMA
            peripherals.DMA_CH3, // RX DMA
            Irqs,
            spi_config,
        );
        let cs_pin = Output::new(unsafe { core::ptr::read(&peripherals.PIN_17) }, Level::High);
        ExclusiveDevice::new(spi_bus, cs_pin, embassy_time::Delay)
    };

    // --- INITIALIZE HARDWARE PERIPHERALS (RP2350 SPECIFIC) ---
    #[cfg(all(feature = "max7456", feature = "rp2350"))]
    let display_ref = {
        // Define SPI hardware transmission speed limits (e.g. 10MHz for MAX7456)
        let mut spi_config = Config::default();
        spi_config.frequency = 10_000_000;
        let spi_irq = interrupt::take!(SPI0);

        // Create the asynchronous SPI instance wrapping hardware SPI0 and DMA Channel 0
        let p = _peripherals;
        let spi = Spi::new(
            peripherals.SPI0,    // Hardware Peripheral Identifier
            peripherals.PIN_18,  // CLK Pin
            peripherals.PIN_19,  // TX (MOSI) Pin
            peripherals.PIN_16,  // RX (MISO) Pin
            peripherals.DMA_CH0, // TX DMA Channel assignment
            peripherals.DMA_CH1, // RX DMA Channel assignment
            spi_irq,
            spi_config,
        );

        // Leak to a safe static reference for the tasks
        let static_spi = SPI_DEVICE_CELL.init(spi);
        let display = DisplayPortMax7456::new(static_spi);

        DISPLAY_PORT_MUTEX_CELL.init(Mutex::new(display))
    };

    // --- INITIALIZE MOCK STUB (HOST PROFILE ENVIRONMENT) ---
    #[allow(unused)]
    #[cfg(not(feature = "max7456"))]
    let display_ref = {
        let raw_display = DisplayPortMock::default();
        DISPLAY_PORT_MUTEX_CELL.init(Mutex::new(raw_display))
    };
    /*     // Allocate a static block of memory for our shared display mutex.
      // 1. Initialize your hardware SPI bus normally

        // 2. Turn it into a &'static mut dyn SpiBus
        let spi_static_ref = SPI_BUS_CELL.init(raw_spi);

        // 3. Construct your driver and shared mutex container
        #[cfg(feature = "max7456")]
        let raw_display = DisplayPortMax7456::new(spi_static_ref);
        #[cfg(not(feature = "max7456"))]
        let raw_display = DisplayPortMock::new();

        let display_ref = SHARED_DISPLAY.init(Mutex::new(raw_display));
    */

    #[cfg(all(feature = "serde", feature = "rp2350"))]
    load_global_configs(peripherals.FLASH).await;
    #[cfg(all(feature = "serde", feature = "std"))]
    load_global_configs().await;

    #[allow(unused_mut)]
    let mut config = GLOBAL_CONFIG.lock().await;

    // ****
    // Initialize the task contexts.
    // ****

    // Initialize the modern storage driver handle matching your u16 Key setup
    let gyro_pid_ctx = GYRO_PID_CTX.init(GyroPidContext {
        flight_control_receiver: flight_control_receiver(),
        gyro_pid_sender: gyro_pid_sender(),
        setpoint_sender: setpoint_sender(),
        fast_config_subscriber: fast_config_subscriber(),
        imu_filters: ImuFilterBank::with_config(config.imu_filter_bank),
        sensor_fusion: MadgwickFilterf32::new(),
        flight_controller: FlightController::new(),
        flight_control_message: FlightControlMessage::new(),
    });

    let imu_ctx = IMU_CTX.init(ImuContext { imu: ImuMock::new(MockImuBus::new(), ImuAxesOrder::XPOS_YPOS_ZPOS) });

    let motor_mixer_ctx = MOTOR_MIXER_CTX.init(MotorMixerContext {
        motor_mixer: MotorMixerQuadXPwm::new(MotorMixerCommon::with_config(config.mixer, config.motor)),
    });

    let flight_control_ctx = FLIGHT_CONTROL_CTX.init(FlightControlContext {
        flight_control_sender: flight_control_sender(),
        config_subscriber: config_subscriber(),
        config_publisher: config_publisher(),
        fast_config_publisher: fast_config_publisher(),
        #[cfg(feature = "autopilot")]
        autopilot_receiver: autopilot_receiver(),
        rates: Rates::new(config.rates),
        rc_modes: RcModes::new(),
        rc_adjustments: RcAdjustments::new(),
    });

    #[cfg(feature = "msp")]
    let msp_ctx = MSP_CTX.init(MspContext {
        msp: Msp::new(),
        fast_config_publisher: fast_config_publisher(),
        config_publisher: config_publisher(),
        #[cfg(feature = "barometer")]
        barometer_subscriber: barometer_subscriber(),
        #[cfg(feature = "battery")]
        battery_subscriber: battery_subscriber(),
        #[cfg(feature = "gps")]
        gps_subscriber: gps_subscriber(),
        #[cfg(feature = "optical_flow")]
        optical_flow_subscriber: optical_flow_subscriber(),
        #[cfg(feature = "rangefinder")]
        rangefinder_subscriber: rangefinder_subscriber(),
        read_buf: [0u8; MSP_READ_BUF_SIZE],
        write_buf: [0u8; MSP_WRITE_BUF_SIZE],
    });

    #[cfg(feature = "blackbox")]
    let blackbox_ctx = {
        //nvs::load_blackbox_config(&mut config.blackbox, &mut flash_driver, config_flash_range.clone());
        use crate::{sensors::SetpointMessage, tasks::gyro_pid_task::gyro_pid_receiver};
        config.blackbox.fields_disabled_mask = FieldSelect::PID_STERM_ROLL
        | FieldSelect::PID_STERM_PITCH
        | FieldSelect::PID_STERM_YAW
        | FieldSelect::PID_KTERM
        //| FieldSelect::PID
        | FieldSelect::RSSI
        | FieldSelect::SETPOINT
        //| FieldSelect::GYRO_UNFILTERED
        | FieldSelect::MOTOR_RPM
        | FieldSelect::BATTERY_VOLTAGE
        | FieldSelect::BATTERY_CURRENT
        | FieldSelect::BAROMETER
        | FieldSelect::RANGEFINDER
        | FieldSelect::ATTITUDE
        | FieldSelect::MAGNETOMETER;

        let mut blackbox = Blackbox::new(config.blackbox);
        blackbox.init();
        BLACKBOX_CTX.init(BlackboxContext {
            gyro_pid_receiver: gyro_pid_receiver(),
            setpoint_receiver: setpoint_receiver(),
            setpoint_message: SetpointMessage::new(),
            #[cfg(feature = "gps")]
            gps_subscriber: gps_subscriber(),
            blackbox,
            buffer: [0u8; 1024],
            pos: 0,
            sd_card: MockSdCard::new("blackbox_log.bbl"),
        })
    };
    #[cfg(all(feature = "blackbox", feature = "rp2350"))]
    let sd_writer_ctx =
        { SD_WRITER_CTX.init(SdWriterContext { buffer: [0u8; 1024], pos: 0, _phantom: core::marker::PhantomData }) };

    #[cfg(feature = "autopilot")]
    let autopilot_ctx: &mut AutopilotContext<'static> = AUTOPILOT_CTX.init(AutopilotContext {
        gyro_pid_receiver: gyro_pid_receiver(),
        setpoint_receiver: setpoint_receiver(),
        flight_control_receiver: flight_control_receiver(),
        autopilot_sender: autopilot_sender(),
        autopilot: Autopilot::new(),
        #[cfg(feature = "barometer")]
        barometer_subscriber: barometer_subscriber(),
        #[cfg(feature = "gps")]
        gps_subscriber: gps_subscriber(),
        #[cfg(feature = "optical_flow")]
        optical_flow_subscriber: optical_flow_subscriber(),
        #[cfg(feature = "rangefinder")]
        rangefinder_subscriber: rangefinder_subscriber(),
    });

    #[cfg(feature = "barometer")]
    let barometer_ctx = BAROMETER_CTX.init(BarometerContext { barometer_publisher: barometer_publisher() });

    #[cfg(feature = "rangefinder")]
    let rangefinder_ctx = RANGEFINDER_CTX.init(RangefinderContext { rangefinder_publisher: rangefinder_publisher() });

    #[cfg(feature = "battery")]
    let battery_ctx = BATTERY_CTX.init(BatteryContext { battery_publisher: battery_publisher() });

    #[cfg(feature = "gps")]
    let gps_ctx = GPS_CTX.init(GpsContext { gps_publisher: gps_publisher(), home: Geodetic::new() });

    #[cfg(feature = "optical_flow")]
    let optical_flow_ctx =
        OPTICAL_FLOW_CTX.init(OpticalFlowContext { optical_flow_publisher: optical_flow_publisher() });

    #[cfg(feature = "osd")]
    let osd_ctx = OSD_CTX.init(OsdContext {
        gyro_pid_receiver: gyro_pid_receiver(),
        setpoint_receiver: setpoint_receiver(),
        #[cfg(feature = "barometer")]
        barometer_subscriber: barometer_subscriber(),
        #[cfg(feature = "battery")]
        battery_subscriber: battery_subscriber(),
        #[cfg(feature = "gps")]
        gps_subscriber: gps_subscriber(),
        #[cfg(feature = "optical_flow")]
        optical_flow_subscriber: optical_flow_subscriber(),
        #[cfg(feature = "rangefinder")]
        rangefinder_subscriber: rangefinder_subscriber(),
        osd: Osd::new(),
    });

    drop(config); // unlocks

    /*
    TODO: gyro_pid on core1 and motor_mixer and radio on high priority interrupt driven spawner
    // Launch Core 1
    unsafe {
        spawn_core1(p.CORE1, &mut CORE1_STACK, core1_entry);
    }    // Create the task tokens and spawn the tasks.
    // 2. Start an InterruptExecutor on Core 0 for 1kHz tasks
    let high_spawner = EXECUTOR_HIGH.start(interrupt::SWI_IRQ_0);
    high_spawner.spawn(motor_mixer_task(motor_mixer_ctx).expect("Failed to create motor mixer task")); // No receiver needed, since it uses a SIGNAL
    high_spawner.spawn(radio_task(radio_ctx).expect("Failed to create radio task"));
    */

    // The gyro_pid task calculates the motor commands, sends them immediately to the motor_mixer task
    // and then updates the GyroPidMessage and sends it.

    // ****
    // Spawn the tasks.
    // ****

    spawner.spawn(gyro_pid_task(gyro_pid_ctx).expect("Failed to create GYRO PID task"));
    spawner.spawn(imu_task(imu_ctx).expect("Failed to create IMU task"));
    spawner.spawn(motor_mixer_task(motor_mixer_ctx).expect("Failed to create MOTOR MIXER task")); // No receiver needed, since it uses a SIGNAL
    spawner.spawn(flight_control_task(flight_control_ctx).expect("Failed to create FLIGHT_CONTROL task"));

    #[cfg(feature = "autopilot")]
    spawner.spawn(autopilot_task(autopilot_ctx).expect("Failed to create AUTOPILOT task"));
    #[cfg(feature = "barometer")]
    spawner.spawn(barometer_task(barometer_ctx).expect("Failed to create BAROMETER task"));
    #[cfg(feature = "battery")]
    spawner.spawn(battery_task(battery_ctx).expect("Failed to create BATTERY task"));
    #[cfg(feature = "blackbox")]
    spawner.spawn(blackbox_task(blackbox_ctx).expect("Failed to create BLACKBOX task"));
    #[cfg(all(feature = "blackbox", feature = "rp2350"))]
    spawner.spawn(sd_writer_task(sd_writer_ctx).expect("Failed to create SD_WRITER task"));
    #[cfg(feature = "gps")]
    spawner.spawn(gps_task(gps_ctx).expect("Failed to create GPS task"));
    #[cfg(feature = "msp")]
    spawner.spawn(msp_task(msp_ctx).expect("Failed to create MSP task"));
    #[cfg(feature = "optical_flow")]
    spawner.spawn(optical_flow_task(optical_flow_ctx).expect("Failed to create OSD task"));
    #[cfg(feature = "osd")]
    spawner.spawn(osd_task(osd_ctx, display_ref).expect("Failed to create OSD task"));
    #[cfg(feature = "rangefinder")]
    spawner.spawn(rangefinder_task(rangefinder_ctx).expect("Failed to create RANGEFINDER task"));
}
