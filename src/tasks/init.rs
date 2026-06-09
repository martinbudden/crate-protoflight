use crate::config::{
    GLOBAL_CONFIG, config_publisher, config_subscriber, fast_config_publisher, fast_config_subscriber,
};

use crate::flight::{FlightControlMessage, FlightController, ImuFilterBank, RcAdjustments};

#[allow(unused)]
use crate::tasks::gyro_pid_task::{gyro_pid_receiver, setpoint_receiver};
use crate::tasks::{
    flight_control_task::{FlightControlContext, flight_control_receiver, flight_control_sender, flight_control_task},
    gyro_pid_task::{GyroPidContext, gyro_pid_sender, gyro_pid_task, setpoint_sender},
    imu_task::{ImuContext, imu_task},
    motor_mixer_task::{MotorMixerContext, motor_mixer_task},
};

#[cfg(feature = "autopilot")]
use crate::{
    autopilot::pilot::Autopilot,
    tasks::{
        autopilot_task::{AutopilotContext, autopilot_task},
        autopilot_task::{autopilot_receiver, autopilot_sender},
    },
};

#[cfg(feature = "barometer")]
use crate::tasks::barometer_task::{
    BarometerContext, barometer_data_publisher, barometer_data_subscriber, barometer_task,
};

#[cfg(feature = "battery")]
use crate::tasks::battery_task::{BatteryContext, battery_data_publisher, battery_data_subscriber, battery_task};

#[cfg(feature = "blackbox")]
use {
    crate::tasks::blackbox_task::{BlackboxContext, blackbox_task},
    blackbox_logger::{Blackbox, FieldSelect, drivers::sd_card::MockSdCard},
};

#[cfg(feature = "gps")]
use crate::{
    gps::Geodetic,
    tasks::gps_task::{GpsContext, gps_data_publisher, gps_data_subscriber, gps_task},
};

#[cfg(feature = "msp")]
use crate::{
    multiwii_serial_protocol::Msp,
    tasks::msp_task::{MSP_READ_BUF_SIZE, MSP_WRITE_BUF_SIZE, MspContext, msp_task},
};

#[cfg(feature = "osd")]
use crate::{
    osd::Osd,
    tasks::osd_task::{OsdContext, osd_task},
};

#[cfg(feature = "rangefinder")]
use crate::tasks::rangefinder_task::{
    RangefinderContext, rangefinder_data_publisher, rangefinder_data_subscriber, rangefinder_task,
};

#[cfg(feature = "serde")]
use crate::tasks::non_volatile_storage as nvs;

#[cfg(feature = "max7456")]
use {crate::display::DisplayPortMax7456, embedded_hal_async::spi::SpiBus};

#[cfg(not(feature = "max7456"))]
use crate::display::DisplayPortMock;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

// --- 1. RASPBERRY PI RP2350 ARCHITECTURE CONFIGURATION ---
#[cfg(all(feature = "max7456", feature = "rp2350"))]
pub type ConcreteSpi = embassy_rp::spi::Spi<'static, embassy_rp::peripherals::SPI0, embassy_rp::spi::Async>;

#[cfg(all(feature = "max7456", feature = "rp2350"))]
pub type SharedDisplay = Mutex<CriticalSectionRawMutex, DisplayPortMax7456<&'static mut ConcreteSpi>>;

// --- 2. HOST ARCHITECTURE TESTING / MOCK CONFIGURATION ---
#[cfg(not(feature = "max7456"))]
pub type SharedDisplay = Mutex<CriticalSectionRawMutex, DisplayPortMock>;

use imu_sensors::{ImuAxesOrder, ImuMock, MockImuBus};
use motor_mixers::{MotorMixerCommon, MotorMixerQuadXPwm};
use radio_controllers::{Rates, RcModes};
use sensor_fusion::MadgwickFilterf32;

#[cfg(feature = "serde")]
use {
    embedded_storage_async::nor_flash::NorFlash,
    sequential_storage::{
        cache::NoCache,
        map::{MapConfig, MapStorage},
    },
};

use static_cell::StaticCell;

use embassy_executor::Spawner;
#[cfg(feature = "multicore")]
use embassy_rp::multicore::{Stack, spawn_core1};

// Core 1 needs its own stack space in RAM
#[cfg(feature = "multicore")]
static mut CORE1_STACK: Stack<4096> = Stack::new();

// --- 1. PC (Host) Build Configuration ---
// If building on your PC (x86_64, Mac, etc.)
// FIX: Replace `_` with `impl embedded_storage_async::nor_flash::NorFlash`
#[cfg(feature = "serde")]
#[cfg(not(target_arch = "arm"))] // If building on your PC (x86_64, Mac, etc.)
#[allow(unused)]
fn init_flash_driver() -> impl embedded_storage_async::nor_flash::NorFlash {
    use embedded_storage_file::{NorMemoryAsync, NorMemoryInFile};

    let path = "pc_mock_flash.nor";
    let capacity_bytes = 1024 * 1024; // Allocate a 1MB virtual flash file

    // 1. Instantiate the synchronous inner file backend with layout properties:
    //    <READ_SIZE, WRITE_SIZE, ERASE_SIZE>
    let inner_sync_nor = NorMemoryInFile::<256, 256, 4096>::new(path, capacity_bytes)
        .expect("Failed to create synchronous mock flash file");

    // 2. FIX: Wrap it using the single-parameter asynchronous wrapper.
    //    We remove the <256, 256, 4096> from NorMemoryAsync to satisfy the 1-generic rule.
    NorMemoryAsync::new(inner_sync_nor)
}

// --- 2. RP2350 (Embedded) Build Configuration ---
#[cfg(target_arch = "arm")] // If building for your physical RP2350 chip
fn init_flash_driver(
    p: embassy_rp::OptionalPeripherals,
) -> embassy_rp::flash::Flash<'static, embassy_rp::peripherals::FLASH, embassy_rp::flash::Async, { 4 * 1024 * 1024 }> {
    use embassy_rp::flash::Flash;
    const FLASH_SIZE_BYTES: usize = 4 * 1024 * 1024;

    Flash::new(p.FLASH, p.DMA_CH0, FLASH_SIZE_BYTES)
}

#[cfg(feature = "serde")]
pub async fn _load_system_configs_task<F>(flash: &mut F, flash_range: core::ops::Range<u32>)
where
    F: NorFlash,
{
    // Initialize the modern storage driver handle matching your u16 Key setup
    let mut storage = MapStorage::new(flash, MapConfig::new(flash_range), NoCache::new());

    let mut config = GLOBAL_CONFIG.lock().await;
    // Execute the loaders sequentially via your clean `lds` namespace shortcut
    nvs::load_imu_filter_bank_config(&mut config.imu_filter_bank, &mut storage).await;
}

/// Create all the contexts for the tasks, and then spawn the tasks.
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
    #[cfg(feature = "gps")]
    static GPS_CTX: StaticCell<GpsContext> = StaticCell::new();
    #[cfg(feature = "msp")]
    static MSP_CTX: StaticCell<MspContext> = StaticCell::new();
    #[cfg(feature = "osd")]
    static OSD_CTX: StaticCell<OsdContext> = StaticCell::new();
    #[cfg(feature = "rangefinder")]
    static RANGEFINDER_CTX: StaticCell<RangefinderContext> = StaticCell::new();

    #[cfg(feature = "rp2350")]
    static SPI_BUS_CELL: StaticCell<ConcreteSpiType> = StaticCell::new();
    static SHARED_DISPLAY_CELL: StaticCell<SharedDisplay> = StaticCell::new();

    // Take ownership of the raw RP2350 hardware peripherals block
    #[cfg(feature = "rp2350")]
    let p = embassy_rp::init(Default::default());
    // --- INITIALIZE HARDWARE PERIPHERALS (RP2350 SPECIFIC) ---
    #[cfg(all(feature = "max7456", feature = "rp2350"))]
    let display_ref = {
        use embassy_rp::spi::{Config, Spi};

        // Define SPI hardware transmission speed limits (e.g. 10MHz for MAX7456)
        let mut spi_config = Config::default();
        spi_config.frequency = 10_000_000;
        let spi_irq = interrupt::take!(SPI0);

        // Create the asynchronous SPI instance wrapping hardware SPI0 and DMA Channel 0
        let raw_spi = Spi::new(
            p.SPI0,    // Hardware Peripheral Identifier
            p.PIN_18,  // CLK Pin
            p.PIN_19,  // TX (MOSI) Pin
            p.PIN_16,  // RX (MISO) Pin
            p.DMA_CH0, // TX DMA Channel assignment
            p.DMA_CH1, // RX DMA Channel assignment
            spi_irq, spi_config,
        );

        // Leak to a safe static reference for the tasks
        let static_spi = SPI_DEVICE_CELL.init(raw_spi);
        let raw_display = DisplayPortMax7456::new(static_spi);

        SHARED_DISPLAY_CELL.init(Mutex::new(raw_display))
    };
    // --- INITIALIZE MOCK STUB (HOST PROFILE ENVIRONMENT) ---
    #[allow(unused)]
    #[cfg(not(feature = "max7456"))]
    let display_ref = {
        let raw_display = DisplayPortMock::new();
        SHARED_DISPLAY_CELL.init(Mutex::new(raw_display))
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

    /*#[cfg(not(target_arch = "arm"))]
    let config_flash_range = 0..1024 * 1024; // Full 1MB simulated range for PC tests

    // Standard Raspberry Pi Pico 2 boards have 4MB of onboard QSPI flash memory.
    #[cfg(target_arch = "arm")]
    let config_flash_range = (4096 - 128) * 1024 .. 4096 * 1024; // Tail end 128KB for chip

    // Initialize our conditional target driver
    #[cfg(not(target_arch = "arm"))]
    let mut flash_driver = init_flash_driver();

    #[cfg(target_arch = "arm")]
    let mut flash_driver = {
        let p = embassy_rp::init(Default::default());
        init_flash_driver(p)
    };

    load_system_configs_task(&mut flash_driver, config_flash_range).await;*/

    env_logger::init();

    #[allow(unused_mut)]
    let mut config = GLOBAL_CONFIG.lock().await;

    // load configs from non-volatile storage.
    //nvs::load_imu_filter_bank_config(&mut config.imu_filter_bank, &mut flash_driver, config_flash_range.clone());
    //nvs::load_imu_filter_bank_config(&mut config.imu_filter_bank, &mut storage).await;

    // ****
    // Initialize the task contexts.
    // ****
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

    //nvs::load_rates_config(&mut config.rates, &mut flash_driver, config_flash_range.clone());
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
        barometer_data_subscriber: barometer_data_subscriber(),
        #[cfg(feature = "battery")]
        battery_data_subscriber: battery_data_subscriber(),
        #[cfg(feature = "gps")]
        gps_data_subscriber: gps_data_subscriber(),
        #[cfg(feature = "rangefinder")]
        rangefinder_data_subscriber: rangefinder_data_subscriber(),
        read_buf: [0u8; MSP_READ_BUF_SIZE],
        write_buf: [0u8; MSP_WRITE_BUF_SIZE],
    });

    #[cfg(feature = "blackbox")]
    let blackbox_ctx = {
        //nvs::load_blackbox_config(&mut config.blackbox, &mut flash_driver, config_flash_range.clone());
        use crate::{flight::FeatureFlags, tasks::gyro_pid_task::gyro_pid_receiver};
        use blackbox_logger::SetpointMessage;
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

        let features = FeatureFlags::INFLIGHT_ACC_CAL | FeatureFlags::RX_SERIAL | FeatureFlags::RSSI_ADC;
        let mut blackbox = Blackbox::new(config.blackbox, features);
        blackbox.init();
        BLACKBOX_CTX.init(BlackboxContext {
            gyro_pid_receiver: gyro_pid_receiver(),
            setpoint_receiver: setpoint_receiver(),
            setpoint_message: SetpointMessage::new(),
            blackbox,
            buffer: [0u8; 1024],
            pos: 0,
            sd_card: MockSdCard::new("blackbox_log.bbl"),
        })
    };

    #[cfg(feature = "autopilot")]
    let autopilot_ctx: &mut AutopilotContext<'static> = AUTOPILOT_CTX.init(AutopilotContext {
        gyro_pid_receiver: gyro_pid_receiver(),
        setpoint_receiver: setpoint_receiver(),
        autopilot_sender: autopilot_sender(),
        autopilot: Autopilot::new(),
        #[cfg(feature = "barometer")]
        barometer_data_subscriber: barometer_data_subscriber(),
        #[cfg(feature = "gps")]
        gps_data_subscriber: gps_data_subscriber(),
        #[cfg(feature = "rangefinder")]
        rangefinder_data_subscriber: rangefinder_data_subscriber(),
    });

    #[cfg(feature = "barometer")]
    let barometer_ctx = BAROMETER_CTX.init(BarometerContext { barometer_data_publisher: barometer_data_publisher() });

    #[cfg(feature = "rangefinder")]
    let rangefinder_ctx =
        RANGEFINDER_CTX.init(RangefinderContext { rangefinder_data_publisher: rangefinder_data_publisher() });

    #[cfg(feature = "battery")]
    let battery_ctx = BATTERY_CTX.init(BatteryContext { battery_data_publisher: battery_data_publisher() });

    #[cfg(feature = "gps")]
    let gps_ctx = GPS_CTX.init(GpsContext { gps_data_publisher: gps_data_publisher(), home: Geodetic::new() });

    #[cfg(feature = "osd")]
    let osd_ctx = OSD_CTX.init(OsdContext {
        gyro_pid_receiver: gyro_pid_receiver(),
        setpoint_receiver: setpoint_receiver(),
        #[cfg(feature = "barometer")]
        barometer_data_subscriber: barometer_data_subscriber(),
        #[cfg(feature = "battery")]
        battery_data_subscriber: battery_data_subscriber(),
        #[cfg(feature = "gps")]
        gps_data_subscriber: gps_data_subscriber(),
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
    #[cfg(feature = "gps")]
    spawner.spawn(gps_task(gps_ctx).expect("Failed to create GPS task"));
    #[cfg(feature = "msp")]
    spawner.spawn(msp_task(msp_ctx).expect("Failed to create MSP task"));
    #[cfg(feature = "osd")]
    spawner.spawn(osd_task(osd_ctx, display_ref).expect("Failed to create OSD task"));
    #[cfg(feature = "rangefinder")]
    spawner.spawn(rangefinder_task(rangefinder_ctx).expect("Failed to create RANGEFINDER task"));
}
