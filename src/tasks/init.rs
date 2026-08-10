use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use static_cell::StaticCell;

use crate::{
    boards::{ImuContext, board_init, imu_context},
    config::GLOBAL_CONFIG,
    tasks::{
        gyro_pid_task::{GyroPidContext, gyro_pid_task},
        imu_task::imu_task,
        motor_mixer_task::{MotorMixerContext, motor_mixer_task},
        rx_task::{RxContext, rx_task},
    },
};

#[cfg(feature = "serde")]
use crate::tasks::non_volatile_storage::load_global_configs;

#[cfg(feature = "autopilot")]
use crate::tasks::autopilot_task::{AutopilotContext, autopilot_task};

#[cfg(feature = "barometer")]
use crate::tasks::barometer_task::{BarometerContext, barometer_task};

#[cfg(feature = "battery")]
use crate::tasks::battery_task::{BatteryContext, battery_task};

#[cfg(feature = "blackbox")]
use crate::tasks::{
    blackbox_task::{BlackboxContext, blackbox_task},
    blackbox_writer_task::{BlackboxWriterContext, blackbox_writer_task},
};

#[cfg(feature = "gps")]
use crate::tasks::gps_task::{GpsContext, gps_task};

#[cfg(feature = "magnetometer")]
use crate::tasks::magnetometer_task::{MagnetometerContext, magnetometer_task};

#[cfg(feature = "msp")]
use crate::tasks::msp_task::{MspContext, msp_task};

#[cfg(feature = "optical_flow")]
use crate::tasks::optical_flow_task::{OpticalFlowContext, optical_flow_task};

#[cfg(feature = "osd")]
use crate::tasks::osd_task::{OsdContext, osd_task};

#[cfg(feature = "rangefinder")]
use crate::tasks::rangefinder_task::{RangefinderContext, rangefinder_task};

#[cfg(feature = "max7456")]
use crate::display::DisplayPortMax7456;

#[cfg(feature = "max7456")]
pub type DisplayPortMax7456Spi = DisplayPortMax7456<DisplaySpi>;
#[cfg(feature = "max7456")]
pub type DisplayPortMutex = Mutex<CriticalSectionRawMutex, DisplayPortMax7456Spi>;

#[cfg(not(feature = "max7456"))]
use crate::display::DisplayPortMock;

// --- 2. HOST ARCHITECTURE TESTING / MOCK CONFIGURATION ---
#[allow(unused)]
#[cfg(not(feature = "max7456"))]
pub type DisplayPortMutex = Mutex<CriticalSectionRawMutex, DisplayPortMock>;

// Core 1 needs its own stack space in RAM
#[cfg(feature = "multicore")]
static mut CORE1_STACK: Stack<4096> = Stack::new();

/// Protoflight initialization, called directly from main.
/// Does the following:
/// 1. Statically allocates all the task contexts (`*_CTX`).
/// 2. Loads the global configuration `GLOBAL_CONFIG`.
/// 3. Initializes the board hardware `board_init`.
/// 4. Initializes all the task contexts using values from `GLOBAL_CONFIG`.
/// 5. spawns all the tasks.
///
/// `panic()`, `.unwrap()` and `.expect()` are allowed during initialization
/// since if anything fails during initialization there is no possibility of recovery
/// and so there is no point continuing.
///
/// Once initialization is complete `panic()`, `.unwrap()` and `.expect()` are NOT allowed.
#[allow(clippy::expect_used)]
#[allow(clippy::too_many_lines)]
pub async fn init(spawner: Spawner) {
    // TODO: put EXECUTOR_CORE1 in a static cell
    #[cfg(feature = "multicore")]
    static EXECUTOR_CORE1: embassy_executor::InterruptExecutor = InterruptExecutor::new();
    //static EXECUTOR_CORE1: StaticCell<Executor> = StaticCell::new();

    // ****
    // Statically allocate the task contexts.
    // ****
    static GYRO_PID_CTX: StaticCell<GyroPidContext> = StaticCell::new();
    static IMU_CTX: StaticCell<ImuContext<crate::boards::BoardImu>> = StaticCell::new();
    static RX_CTX: StaticCell<RxContext> = StaticCell::new();
    static MOTOR_MIXER_CTX: StaticCell<MotorMixerContext> = StaticCell::new();

    #[cfg(feature = "autopilot")]
    static AUTOPILOT_CTX: StaticCell<AutopilotContext> = StaticCell::new();
    #[cfg(feature = "barometer")]
    static BAROMETER_CTX: StaticCell<BarometerContext> = StaticCell::new();
    #[cfg(feature = "battery")]
    static BATTERY_CTX: StaticCell<BatteryContext> = StaticCell::new();
    #[cfg(feature = "blackbox")]
    static BLACKBOX_CTX: StaticCell<BlackboxContext> = StaticCell::new();
    #[cfg(feature = "blackbox")]
    static BLACKBOX_WRITER_CTX: StaticCell<BlackboxWriterContext> = StaticCell::new();
    #[cfg(feature = "gps")]
    static GPS_CTX: StaticCell<GpsContext> = StaticCell::new();
    #[cfg(feature = "magnetometer")]
    static MAGNETOMETER_CTX: StaticCell<MagnetometerContext> = StaticCell::new();
    #[cfg(feature = "msp")]
    static MSP_CTX: StaticCell<MspContext> = StaticCell::new();
    #[cfg(feature = "optical_flow")]
    static OPTICAL_FLOW_CTX: StaticCell<OpticalFlowContext> = StaticCell::new();
    #[cfg(feature = "osd")]
    static OSD_CTX: StaticCell<OsdContext> = StaticCell::new();
    #[cfg(feature = "rangefinder")]
    static RANGEFINDER_CTX: StaticCell<RangefinderContext> = StaticCell::new();
    #[allow(unused)]
    static DISPLAY_PORT_MUTEX_CELL: StaticCell<DisplayPortMutex> = StaticCell::new();

    // Initialize env_logger for logging to stdout on desktop platforms.
    // This connects the logger to the terminal and polls the environment variables.
    #[cfg(feature = "std")]
    env_logger::init();

    // **** Load the GLOBAL_CONFIGs
    #[cfg(all(feature = "serde", feature = "rp2350"))]
    load_global_configs(board.flash).await;
    #[cfg(all(feature = "serde", feature = "std"))]
    load_global_configs().await;
    let config = GLOBAL_CONFIG.lock().await;

    // **** GET THE DEVICES FROM THE BOARD SUPPORT PACKAGE
    let board = board_init(config.imu_device.axis_order);

    #[rustfmt::skip]
    #[cfg(feature = "osd")]
    let display_ref = {
        #[cfg(feature = "max7456")] { DISPLAY_PORT_MUTEX_CELL.init(Mutex::new(DisplayPortMax7456Spi::new(aux_pio_spi))) }
        #[cfg(not(feature = "max7456"))] { DISPLAY_PORT_MUTEX_CELL.init(Mutex::new(DisplayPortMock::default())) }
    };

    // ****
    // Initialize the task contexts.
    // ****

    // Initialize the modern storage driver handle matching your u16 Key setup
    #[rustfmt::skip]
    let gyro_pid_ctx = GYRO_PID_CTX.init(GyroPidContext::new(
        config.imu_filter_bank,
        #[cfg(feature = "rpm_filters")] config.rpm_notch_filter_bank,
        #[cfg(feature = "rpm_filters")] 0.001,
    ));

    let imu_ctx = IMU_CTX.init(imu_context(board.imu));

    #[rustfmt::skip]
    let motor_mixer_ctx = MOTOR_MIXER_CTX.init(MotorMixerContext::new(
        config.mixer,
        config.motor,
        board.motor_driver.expect("motor driver fail"),
        #[cfg(feature = "rpm_filters")] config.rpm_notch_filter_bank,
        #[cfg(feature = "rpm_filters")] 0.001
    ));

    let rx_ctx = RX_CTX.init(RxContext::new(config.rates));

    #[cfg(feature = "msp")]
    let msp_ctx = MSP_CTX.init(MspContext::new());

    #[cfg(feature = "blackbox")]
    let blackbox_ctx = { BLACKBOX_CTX.init(BlackboxContext::new(config.blackbox)) };
    #[cfg(all(feature = "blackbox", feature = "rp2350"))]
    let blackbox_writer_ctx = { BLACKBOX_WRITER_CTX.init(BlackboxWriterContext::new(board.sdcard_spi.unwrap())) };
    #[cfg(all(feature = "blackbox", feature = "std"))]
    let blackbox_writer_ctx = BLACKBOX_WRITER_CTX.init(BlackboxWriterContext::new());

    #[cfg(feature = "autopilot")]
    let autopilot_ctx: &mut AutopilotContext = AUTOPILOT_CTX.init(AutopilotContext::new());

    #[cfg(feature = "barometer")]
    let barometer_ctx = BAROMETER_CTX.init(BarometerContext::new());

    #[cfg(feature = "battery")]
    let battery_ctx = BATTERY_CTX.init(BatteryContext::new());

    #[cfg(feature = "gps")]
    let gps_ctx = GPS_CTX.init(GpsContext::new());

    #[cfg(feature = "magnetometer")]
    let magnetometer_ctx = MAGNETOMETER_CTX.init(MagnetometerContext::new());

    #[cfg(feature = "optical_flow")]
    let optical_flow_ctx = OPTICAL_FLOW_CTX.init(OpticalFlowContext::new());

    #[cfg(feature = "osd")]
    let osd_ctx = {
        let display_supports_background_layer = true;
        OSD_CTX.init(OsdContext::new(display_supports_background_layer))
    };

    #[cfg(feature = "rangefinder")]
    let rangefinder_ctx = RANGEFINDER_CTX.init(RangefinderContext::new());

    drop(config); // unlocks

    /*
    TODO: for raspberry pi pico put gyro_pid on core1 and motor_mixer and radio on high priority interrupt driven spawner
        // 1. Launch Core 1
        unsafe { spawn_core1(p.CORE1, &mut CORE1_STACK, core1_entry); }
        // 2. Start an InterruptExecutor on Core 0 for the 1kHz tasks, ie the motor_mixer and rx tasks.
        let high_spawner = EXECUTOR_HIGH.start(interrupt::SWI_IRQ_0);
        high_spawner.spawn(motor_mixer_task(motor_mixer_ctx).expect("Failed to create motor mixer task")); // No receiver needed, since it uses a SIGNAL
        high_spawner.spawn(rx_task(rx_ctx).expect("Failed to create radio task"));
    */

    // ****
    // Spawn the tasks.
    // ****

    spawner.spawn(gyro_pid_task(gyro_pid_ctx).expect("Failed to create GYRO PID task"));
    spawner.spawn(imu_task(imu_ctx).expect("Failed to create IMU task"));
    spawner.spawn(motor_mixer_task(motor_mixer_ctx).expect("Failed to create MOTOR MIXER task")); // No receiver needed, since it uses a SIGNAL
    spawner.spawn(rx_task(rx_ctx).expect("Failed to create RX task"));

    #[cfg(feature = "autopilot")]
    spawner.spawn(autopilot_task(autopilot_ctx).expect("Failed to create AUTOPILOT task"));
    #[cfg(feature = "barometer")]
    spawner.spawn(barometer_task(barometer_ctx).expect("Failed to create BAROMETER task"));
    #[cfg(feature = "battery")]
    spawner.spawn(battery_task(battery_ctx).expect("Failed to create BATTERY task"));

    #[cfg(feature = "blackbox")]
    spawner.spawn(blackbox_task(blackbox_ctx).expect("Failed to create BLACKBOX task"));
    #[cfg(feature = "blackbox")]
    spawner.spawn(blackbox_writer_task(blackbox_writer_ctx).expect("Failed to create BLACKBOX_WRITER task"));
    #[cfg(feature = "gps")]
    spawner.spawn(gps_task(gps_ctx).expect("Failed to create GPS task"));
    #[cfg(feature = "magnetometer")]
    spawner.spawn(magnetometer_task(magnetometer_ctx).expect("Failed to create MAGNETOMETER task"));
    #[cfg(feature = "msp")]
    spawner.spawn(msp_task(msp_ctx).expect("Failed to create MSP task"));
    #[cfg(feature = "optical_flow")]
    spawner.spawn(optical_flow_task(optical_flow_ctx).expect("Failed to create OSD task"));
    #[cfg(feature = "osd")]
    spawner.spawn(osd_task(osd_ctx, display_ref).expect("Failed to create OSD task"));
    #[cfg(feature = "rangefinder")]
    spawner.spawn(rangefinder_task(rangefinder_ctx).expect("Failed to create RANGEFINDER task"));
}
