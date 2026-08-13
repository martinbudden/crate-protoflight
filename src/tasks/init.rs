use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use static_cell::StaticCell;

use crate::{
    boards::{BoardInit, board_init},
    config::GLOBAL_CONFIG,
};

#[cfg(feature = "serde")]
use crate::tasks::non_volatile_storage::load_global_configs;

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
/// 1. Loads the global configuration `GLOBAL_CONFIG`.
/// 2. Initializes the board hardware `board_init`.
/// 3. Initializes all the task contexts using values from `GLOBAL_CONFIG`.
/// 4. Drops `GLOBAL_CONFIG`.
/// 5. spawns all the tasks.
///
/// `panic()`, `.unwrap()` and `.expect()` are allowed during initialization
/// since if anything fails during initialization there is no possibility of recovery
/// and so there is no point continuing.
///
/// Once initialization is complete `panic()`, `.unwrap()` and `.expect()` are NOT allowed.
///
#[allow(clippy::too_many_lines)]
#[allow(clippy::expect_used)]
pub async fn init(spawner: Spawner) {
    use crate::tasks;

    // TODO: put EXECUTOR_CORE1 in a static cell
    #[cfg(feature = "multicore")]
    static EXECUTOR_CORE1: embassy_executor::InterruptExecutor = InterruptExecutor::new();
    //static EXECUTOR_CORE1: StaticCell<Executor> = StaticCell::new();

    #[allow(unused)]
    static DISPLAY_PORT_MUTEX_CELL: StaticCell<DisplayPortMutex> = StaticCell::new();

    // Initialize env_logger for logging to stdout on desktop platforms.
    // This connects the logger to the terminal and polls the environment variables.
    #[cfg(feature = "std")]
    env_logger::init();

    // **** Load and lock the GLOBAL_CONFIGs
    #[cfg(all(feature = "serde", feature = "rp2350"))]
    load_global_configs(board_flash()).await;
    #[cfg(all(feature = "serde", feature = "std"))]
    load_global_configs().await;
    let config = GLOBAL_CONFIG.lock().await;

    // **** GET THE DEVICES FROM THE BOARD SUPPORT PACKAGE

    #[allow(clippy::panic)]
    let Ok(board) = board_init(BoardInit {
        axis_order: config.imu_device.axis_order,
        radio_type: config.rx.serial_rx_provider,

        #[cfg(feature = "barometer")]
        barometer_type: config.barometer.hardware,
        #[cfg(not(feature = "barometer"))]
        barometer_type: crate::barometer_sensors::BarometerType::None,

        #[cfg(feature = "magnetometer")]
        magnetometer_type: config.magnetometer.hardware,
        #[cfg(not(feature = "magnetometer"))]
        magnetometer_type: crate::magnetometer_sensors::MagnetometerType::None,

        #[cfg(feature = "rangefinder")]
        rangefinder_type: config.rangefinder.hardware,
        #[cfg(not(feature = "rangefinder"))]
        rangefinder_type: crate::rangefinder_sensors::RangefinderType::None,

        #[cfg(feature = "optical_flow")]
        optical_flow_type: config.optical_flow.hardware,
        #[cfg(not(feature = "optical_flow"))]
        optical_flow_type: crate::optical_flow_sensors::OpticalFlowType::None,
    }) else {
        panic!("board_init failed");
    };

    #[rustfmt::skip]
    #[cfg(feature = "osd")]
    let display_ref = {
        #[cfg(feature = "max7456")] { DISPLAY_PORT_MUTEX_CELL.init(Mutex::new(DisplayPortMax7456Spi::new(aux_pio_spi))) }
        #[cfg(not(feature = "max7456"))] { DISPLAY_PORT_MUTEX_CELL.init(Mutex::new(DisplayPortMock::default())) }
    };

    // ****
    // Initialize the task contexts.
    // ****

    #[rustfmt::skip]
    let gyro_pid_ctx = tasks::gyro_pid::init(
        config.imu_filter_bank,
        #[cfg(feature = "rpm_filters")] config.rpm_notch_filter_bank,
        #[cfg(feature = "rpm_filters")] 0.001,
    );

    // Initialize the IMU task context with the IMU provided by the Board Support Package.
    let imu_ctx = tasks::imu::init(board.imu);

    // Initialize the motor mixer task context with the motor driver provided by the Board Support Package.
    #[rustfmt::skip]
    let motor_mixer_ctx = tasks::motor_mixer::init(
        config.mixer,
        config.motor,
        board.motor_driver,
        #[cfg(feature = "rpm_filters")] config.rpm_notch_filter_bank,
        #[cfg(feature = "rpm_filters")] 0.001
    );

    // TODO: Initialize the receiver task context with the UART provided by the Board Support Package.
    let rx_ctx = tasks::rx::init(board.radio, config.rates);

    // TODO: Initialize the MSP task context with the UART provided by the Board Support Package.
    #[cfg(feature = "msp")]
    let msp_ctx = tasks::msp::init();

    // TODO: Initialize the blackbox task context with the ... provided by the Board Support Package.
    #[cfg(feature = "blackbox")]
    let blackbox_ctx = tasks::blackbox::init(config.blackbox);
    #[cfg(feature = "blackbox")]
    let blackbox_writer_ctx = Some(tasks::blackbox_writer::init());

    #[cfg(feature = "autopilot")]
    let autopilot_ctx = tasks::autopilot::init();

    #[cfg(feature = "barometer")]
    let barometer_ctx = board.barometer.map(tasks::barometer::init);

    /*
    Alternatively
    #[cfg(feature = "barometer")]
    let barometer_ctx = match board.barometer {
        Some(barometer) => Some(tasks::barometer::init(barometer)),
        None => None,
    };*/

    #[cfg(feature = "battery")]
    let battery_ctx = tasks::battery::init();

    #[cfg(feature = "gps")]
    let gps_ctx = Some(tasks::gps::init());

    #[cfg(feature = "magnetometer")]
    let magnetometer_ctx = board.magnetometer.map(tasks::magnetometer::init);

    #[cfg(feature = "optical_flow")]
    let optical_flow_ctx = board.optical_flow.map(tasks::optical_flow::init);

    #[cfg(feature = "osd")]
    let osd_ctx = {
        let display_supports_background_layer = true;
        Some(tasks::osd::init(display_supports_background_layer))
    };

    #[cfg(feature = "rangefinder")]
    let rangefinder_ctx = board.rangefinder.map(tasks::rangefinder::init);

    // **** UnLock the GLOBAL_CONFIGs
    drop(config);

    /*
    TODO: for raspberry pi pico put gyro_pid on core1 and motor_mixer and radio on high priority interrupt driven spawner
        // 1. Launch Core 1
        unsafe { spawn_core1(p.CORE1, &mut CORE1_STACK, core1_entry); }
        // 2. Start an InterruptExecutor on Core 0 for the 1kHz tasks, ie the motor_mixer and rx tasks.
        let high_spawner = EXECUTOR_HIGH.start(interrupt::SWI_IRQ_0);
        high_spawner.spawn(motor_mixer(motor_mixer_ctx).expect("Failed to create motor mixer task")); // No receiver needed, since it uses a SIGNAL
        high_spawner.spawn(rx(rx_ctx).expect("Failed to create radio task"));
    */

    // ****
    // Spawn the tasks.
    // ****

    // The four mandatory tasks.
    spawner.spawn(tasks::gyro_pid::run(gyro_pid_ctx).expect("Failed to create GYRO PID task"));
    spawner.spawn(tasks::imu::run(imu_ctx).expect("Failed to create IMU task"));
    spawner.spawn(tasks::motor_mixer::run(motor_mixer_ctx).expect("Failed to create MOTOR MIXER task"));
    spawner.spawn(tasks::rx::run(rx_ctx).expect("Failed to create RX task"));

    // The optional tasks.
    #[cfg(feature = "autopilot")]
    spawner.spawn(tasks::autopilot::run(autopilot_ctx).expect("Failed to create AUTOPILOT task"));

    #[cfg(feature = "barometer")]
    if let Some(barometer_ctx) = barometer_ctx {
        spawner.spawn(tasks::barometer::run(barometer_ctx).expect("Failed to create BAROMETER task"));
    }

    #[cfg(feature = "battery")]
    spawner.spawn(tasks::battery::run(battery_ctx).expect("Failed to create BATTERY task"));
    #[cfg(feature = "blackbox")]
    spawner.spawn(tasks::blackbox::run(blackbox_ctx).expect("Failed to create BLACKBOX task"));
    #[cfg(feature = "blackbox")]
    if let Some(blackbox_writer_ctx) = blackbox_writer_ctx {
        spawner.spawn(tasks::blackbox_writer::run(blackbox_writer_ctx).expect("Failed to create BLACKBOX_WRITER task"));
    }
    #[cfg(feature = "gps")]
    if let Some(gps_ctx) = gps_ctx {
        spawner.spawn(tasks::gps::run(gps_ctx).expect("Failed to create GPS task"));
    }
    #[cfg(feature = "magnetometer")]
    if let Some(magnetometer_ctx) = magnetometer_ctx {
        spawner.spawn(tasks::magnetometer::run(magnetometer_ctx).expect("Failed to create MAGNETOMETER task"));
    }
    #[cfg(feature = "msp")]
    spawner.spawn(tasks::msp::run(msp_ctx).expect("Failed to create MSP task"));
    #[cfg(feature = "optical_flow")]
    if let Some(optical_flow_ctx) = optical_flow_ctx {
        spawner.spawn(tasks::optical_flow::run(optical_flow_ctx).expect("Failed to create OSD task"));
    }
    #[cfg(feature = "osd")]
    if let Some(osd_ctx) = osd_ctx {
        spawner.spawn(tasks::osd::run(osd_ctx, display_ref).expect("Failed to create OSD task"));
    }
    #[cfg(feature = "rangefinder")]
    if let Some(rangefinder_ctx) = rangefinder_ctx {
        spawner.spawn(tasks::rangefinder::run(rangefinder_ctx).expect("Failed to create RANGEFINDER task"));
    }
}
