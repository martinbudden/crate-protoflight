#![allow(unused)]
use crate::config::{
    GLOBAL_CONFIG, config_publisher, config_subscriber, fast_config_publisher, fast_config_subscriber,
};
use crate::sensor_data::{
    fast_sensor_data_publisher, fast_sensor_data_subscriber, sensor_data_publisher, sensor_data_subscriber,
};

use crate::flight::{FlightController, ImuFilterBank, RcAdjustments};

use crate::tasks::dispatch::{gyro_pid_receiver, gyro_pid_sender, setpoint_receiver, setpoint_sender};
use crate::tasks::non_volatile_storage as nvs;
use crate::tasks::{GYRO_CTX, GyroPidContext, gyro_pid_task};
use crate::tasks::{MOTOR_MIXER_CTX, MotorMixerContext, motor_mixer_task};
use crate::tasks::{
    radio_task::{radio_receiver, radio_sender},
    {RADIO_CTX, RadioContext, radio_task},
};

#[cfg(feature = "autopilot")]
use crate::{
    autopilot::pilot::Autopilot,
    tasks::{
        AUTOPILOT_CTX, AutopilotContext, autopilot_task,
        radio_task::{autopilot_receiver, autopilot_sender},
    },
};

#[cfg(feature = "barometer")]
use crate::tasks::{BAROMETER_CTX, BarometerContext, barometer_task};

#[cfg(feature = "blackbox")]
use {
    crate::tasks::{BLACKBOX_CTX, BlackboxContext, blackbox_task},
    blackbox_logger::{Blackbox, FieldSelect, drivers::sd_card::MockSdCard},
};

#[cfg(feature = "gps")]
use crate::{
    gps::Geodetic,
    tasks::{GPS_CTX, GpsContext, gps_task},
};

#[cfg(feature = "msp")]
use crate::{
    multiwii_serial_protocol::Msp,
    tasks::{MSP_CTX, MSP_READ_BUF_SIZE, MSP_WRITE_BUF_SIZE, MspContext, msp_task},
};

#[cfg(feature = "osd")]
use crate::{
    osd::Osd,
    tasks::{OSD_CTX, OsdContext, osd_task},
};

use imu_sensors::ImuAxesOrder;
use imu_sensors::{ImuMock, MockImuBus};
//use sequential_storage::cache::{self, NoCache};
//use sequential_storage::map::SerializationError;

use motor_mixers::{MotorMixerCommon, MotorMixerQuadXPwm};
use sensor_fusion::MadgwickFilterf32;

use embedded_storage_async::nor_flash::NorFlash;
use radio_controllers::{RadioControlMessage, Rates, RcModes};
use sequential_storage::cache::NoCache;
use sequential_storage::map::{MapConfig, MapStorage};

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
    env_logger::init();
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

    let mut config = GLOBAL_CONFIG.lock().await;

    // load configs from non-volatile storage.
    //nvs::load_imu_filter_bank_config(&mut config.imu_filter_bank, &mut flash_driver, config_flash_range.clone());
    //nvs::load_imu_filter_bank_config(&mut config.imu_filter_bank, &mut storage).await;

    let gyro_pid_ctx = GYRO_CTX.init(GyroPidContext {
        radio_receiver: radio_receiver(),
        gyro_pid_sender: gyro_pid_sender(),
        setpoint_sender: setpoint_sender(),
        fast_config_subscriber: fast_config_subscriber(),
        fast_sensor_data_subscriber: fast_sensor_data_subscriber(),
        imu: ImuMock::new(MockImuBus::new(), ImuAxesOrder::XPOS_YPOS_ZPOS),
        imu_filters: ImuFilterBank::with_config(config.imu_filter_bank),
        sensor_fusion: MadgwickFilterf32::new(),
        flight_controller: FlightController::new(),
        radio_control_message: RadioControlMessage::new(),
    });

    let motor_mixer_ctx = MOTOR_MIXER_CTX.init(MotorMixerContext {
        motor_mixer: MotorMixerQuadXPwm::new(MotorMixerCommon::with_config(config.mixer, config.motor)),
    });

    //nvs::load_rates_config(&mut config.rates, &mut flash_driver, config_flash_range.clone());
    let radio_ctx = RADIO_CTX.init(RadioContext {
        radio_sender: radio_sender(),
        #[cfg(feature = "autopilot")]
        autopilot_receiver: autopilot_receiver(),
        config_subscriber: config_subscriber(),
        config_publisher: config_publisher(),
        fast_config_publisher: fast_config_publisher(),
        rates: Rates::new(config.rates),
        rc_modes: RcModes::new(),
        rc_adjustments: RcAdjustments::new(),
    });

    #[cfg(feature = "msp")]
    let msp_ctx = MSP_CTX.init(MspContext {
        msp: Msp::new(),
        fast_config_publisher: fast_config_publisher(),
        config_publisher: config_publisher(),
        sensor_data_subscriber: sensor_data_subscriber(),
        read_buf: [0u8; MSP_READ_BUF_SIZE],
        write_buf: [0u8; MSP_WRITE_BUF_SIZE],
    });

    #[cfg(feature = "blackbox")]
    let blackbox_ctx = {
        //nvs::load_blackbox_config(&mut config.blackbox, &mut flash_driver, config_flash_range.clone());

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
        sensor_data_subscriber: sensor_data_subscriber(),
        gyro_pid_receiver: gyro_pid_receiver(),
        setpoint_receiver: setpoint_receiver(),
        autopilot_sender: autopilot_sender(),
        autopilot: Autopilot::new(),
    });

    #[cfg(feature = "barometer")]
    let barometer_ctx = BAROMETER_CTX.init(BarometerContext { sensor_data_publisher: sensor_data_publisher() });

    #[cfg(feature = "gps")]
    let gps_ctx = GPS_CTX.init(GpsContext {
        fast_sensor_data_publisher: fast_sensor_data_publisher(),
        sensor_data_publisher: sensor_data_publisher(),
        home: Geodetic::new(),
    });

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
    spawner.spawn(gyro_pid_task(gyro_pid_ctx).expect("Failed to create GYRO PID task"));
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
}
