//#![allow(unused)]
use crate::config::{
    GLOBAL_CONFIG, config_publisher, config_subscriber, fast_config_publisher, fast_config_subscriber,
};

use crate::flight::{FlightController, ImuFilterBank, RcAdjustments};

#[allow(unused)]
use crate::tasks::gyro_pid_task::{gyro_pid_receiver, setpoint_receiver};
#[allow(unused)]
use crate::tasks::non_volatile_storage as nvs;
use crate::tasks::{
    gyro_pid_task::{GyroPidContext, gyro_pid_sender, gyro_pid_task, setpoint_sender},
    imu_task::{ImuContext, imu_task},
    motor_mixer_task::{MotorMixerContext, motor_mixer_task},
    radio_task::{RadioContext, radio_task},
    radio_task::{radio_receiver, radio_sender},
};

#[cfg(feature = "autopilot")]
use crate::{
    autopilot::pilot::Autopilot,
    tasks::{
        autopilot_task::{AutopilotContext, autopilot_task},
        autopilot_task::{autopilot_receiver, autopilot_sender},
        barometer_task::barometer_data_subscriber,
    },
};

#[cfg(feature = "barometer")]
use crate::tasks::barometer_task::{BarometerContext, barometer_data_publisher, barometer_task};

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

use imu_sensors::{ImuAxesOrder, ImuMock, MockImuBus};
use motor_mixers::{MotorMixerCommon, MotorMixerQuadXPwm};
use radio_controllers::{RadioControlMessage, Rates, RcModes};
use sensor_fusion::MadgwickFilterf32;

use embedded_storage_async::nor_flash::NorFlash;
use sequential_storage::{
    cache::NoCache,
    map::{MapConfig, MapStorage},
};
//use sequential_storage::map::SerializationError;

use embassy_executor::Spawner;
#[cfg(feature = "multicore")]
use embassy_rp::multicore::{Stack, spawn_core1};

// Core 1 needs its own stack space in RAM
#[cfg(feature = "multicore")]
static mut CORE1_STACK: Stack<4096> = Stack::new();

// --- 1. PC (Host) Build Configuration ---
#[cfg(not(target_arch = "arm"))]
// If building on your PC (x86_64, Mac, etc.)
// FIX: Replace `_` with `impl embedded_storage_async::nor_flash::NorFlash`
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
    //    We remove the <256, 256, 4096> from NorMemoryAsync to satisfy the 1-generic rule!
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

#[allow(unused)]
pub async fn load_system_configs_task<F>(flash: &mut F, flash_range: core::ops::Range<u32>)
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
    static IMU_CTX: static_cell::StaticCell<ImuContext> = static_cell::StaticCell::new();
    static GYRO_PID_CTX: static_cell::StaticCell<GyroPidContext> = static_cell::StaticCell::new();
    static RADIO_CTX: static_cell::StaticCell<RadioContext> = static_cell::StaticCell::new();
    static MOTOR_MIXER_CTX: static_cell::StaticCell<MotorMixerContext> = static_cell::StaticCell::new();

    #[cfg(feature = "autopilot")]
    static AUTOPILOT_CTX: static_cell::StaticCell<AutopilotContext> = static_cell::StaticCell::new();
    #[cfg(feature = "barometer")]
    static BAROMETER_CTX: static_cell::StaticCell<BarometerContext> = static_cell::StaticCell::new();
    #[cfg(feature = "blackbox")]
    static BLACKBOX_CTX: static_cell::StaticCell<BlackboxContext> = static_cell::StaticCell::new();
    #[cfg(feature = "gps")]
    static GPS_CTX: static_cell::StaticCell<GpsContext> = static_cell::StaticCell::new();
    #[cfg(feature = "msp")]
    static MSP_CTX: static_cell::StaticCell<MspContext> = static_cell::StaticCell::new();
    #[cfg(feature = "osd")]
    static OSD_CTX: static_cell::StaticCell<OsdContext> = static_cell::StaticCell::new();
    #[cfg(feature = "rangefinder")]
    static RANGEFINDER_CTX: static_cell::StaticCell<RangefinderContext> = static_cell::StaticCell::new();

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
        radio_receiver: radio_receiver(),
        gyro_pid_sender: gyro_pid_sender(),
        setpoint_sender: setpoint_sender(),
        fast_config_subscriber: fast_config_subscriber(),
        imu_filters: ImuFilterBank::with_config(config.imu_filter_bank),
        sensor_fusion: MadgwickFilterf32::new(),
        flight_controller: FlightController::new(),
        radio_control_message: RadioControlMessage::new(),
    });

    let imu_ctx = IMU_CTX.init(ImuContext { imu: ImuMock::new(MockImuBus::new(), ImuAxesOrder::XPOS_YPOS_ZPOS) });

    let motor_mixer_ctx = MOTOR_MIXER_CTX.init(MotorMixerContext {
        motor_mixer: MotorMixerQuadXPwm::new(MotorMixerCommon::with_config(config.mixer, config.motor)),
    });

    //nvs::load_rates_config(&mut config.rates, &mut flash_driver, config_flash_range.clone());
    let radio_ctx = RADIO_CTX.init(RadioContext {
        radio_sender: radio_sender(),
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
        use crate::tasks::gyro_pid_task::gyro_pid_receiver;
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
        let mut blackbox = Blackbox::new(config.blackbox);
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

    #[cfg(feature = "gps")]
    let gps_ctx = GPS_CTX.init(GpsContext { gps_data_publisher: gps_data_publisher(), home: Geodetic::new() });

    #[cfg(feature = "osd")]
    let osd_ctx = OSD_CTX.init(OsdContext {
        gyro_pid_receiver: gyro_pid_receiver(),
        setpoint_receiver: setpoint_receiver(),
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
    spawner.spawn(radio_task(radio_ctx).expect("Failed to create RADIO task"));

    #[cfg(feature = "autopilot")]
    spawner.spawn(autopilot_task(autopilot_ctx).expect("Failed to create AUTOPILOT task"));
    #[cfg(feature = "barometer")]
    spawner.spawn(barometer_task(barometer_ctx).expect("Failed to create BAROMETER task"));
    #[cfg(feature = "blackbox")]
    spawner.spawn(blackbox_task(blackbox_ctx).expect("Failed to create BLACKBOX task"));
    #[cfg(feature = "gps")]
    spawner.spawn(gps_task(gps_ctx).expect("Failed to create GPS task"));
    #[cfg(feature = "msp")]
    spawner.spawn(msp_task(msp_ctx).expect("Failed to create MSP task"));
    #[cfg(feature = "osd")]
    spawner.spawn(osd_task(osd_ctx).expect("Failed to create OSD task"));
    #[cfg(feature = "rangefinder")]
    spawner.spawn(rangefinder_task(rangefinder_ctx).expect("Failed to create RANGEFINDER task"));
}
