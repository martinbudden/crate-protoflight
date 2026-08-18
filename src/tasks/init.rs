use embassy_executor::Spawner;

use crate::{
    boards::{BoardInit, board_hardware},
    config::GLOBAL_CONFIG,
};

/// Protoflight initialization, called directly from main.
/// Does the following:
/// 1. Loads the global configuration `GLOBAL_CONFIG` from non-volatile-storage.
/// 2. Locks `GLOBAL_CONFIG`.
/// 3. Construct the board hardware given the board type and the configuration `board_hardware`.
/// 5. Initializes all the task contexts using values from `GLOBAL_CONFIG`.
/// 6. Unlocks `GLOBAL_CONFIG`.
/// 7. Spawns the realtime tasks.
/// 8. Spawns any background tasks that have been configured to run.
///
/// `panic()`, `.unwrap()` and `.expect()` are allowed during initialization
/// since if anything fails during initialization there is no possibility of recovery
/// and so there is no point continuing.
///
/// Once initialization is complete `panic()`, `.unwrap()` and `.expect()` are NOT allowed.
///
/// This function is quite long, but it is long for a good reason, and its organization is clear.
///
/// Replacing this one understandable 250-line function with (say) five 50-line functions would mean
/// you'd have to jump around to understand startup and it would reduce clarity.
///
#[allow(clippy::too_many_lines)]
pub async fn init(spawner: Spawner) {
    use crate::tasks;

    // Initialize env_logger for logging to stdout on desktop platforms.
    // This connects the logger to the terminal and polls the environment variables.
    #[cfg(feature = "std")]
    env_logger::init();

    // ==================================================
    // Load the GLOBAL_CONFIGs from non-volatile storage.
    // ==================================================

    #[cfg(all(feature = "serde", feature = "rp2350"))]
    tasks::non_volatile_storage::load_global_configs(board_flash()).await;
    #[cfg(all(feature = "serde", feature = "std"))]
    let _err = tasks::non_volatile_storage::load_global_configs(tasks::non_volatile_storage::init_flash_driver()).await;

    // ==================================================
    // Lock the GLOBAL_CONFIGs.
    // ==================================================

    // The lock is held until all the task context have been initialized.
    // This is perfectly fine since no tasks have been spawned yet, so nothing will be blocked.
    let config = GLOBAL_CONFIG.lock().await;

    // ==================================================
    // Get the devices from the board support package
    // ==================================================

    let board_init = BoardInit {
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

        #[cfg(feature = "gps")]
        gps_provider: config.gps.provider,
        #[cfg(not(feature = "gps"))]
        gps_provider: crate::gps::GpsProvider::None,

        #[cfg(feature = "rangefinder")]
        rangefinder_type: config.rangefinder.hardware,
        #[cfg(not(feature = "rangefinder"))]
        rangefinder_type: crate::rangefinder_sensors::RangefinderType::None,

        #[cfg(feature = "optical_flow")]
        optical_flow_type: config.optical_flow.hardware,
        #[cfg(not(feature = "optical_flow"))]
        optical_flow_type: crate::optical_flow_sensors::OpticalFlowType::None,
    };

    #[allow(clippy::panic)]
    let Ok(hardware) = board_hardware(board_init) else {
        panic!("board_init failed");
    };

    // ==================================================
    // Initialize the display port mutex.
    // ==================================================

    // The display port mutex guards shared access to the display port.
    // It is currently only used by the OSD, but will also be used with the Context Menu System (CMS) when it is implemented.
    #[cfg(any(feature = "osd", feature = "cms"))]
    let display_port_mutex = crate::display::display_port_mutex_init();

    // ==================================================
    // Initialize the task contexts.
    // ==================================================

    #[rustfmt::skip]
    let gyro_pid_ctx = tasks::gyro_pid::init(
        config.imu_filter_bank,
        #[cfg(feature = "rpm_filters")] config.rpm_notch_filter_bank,
        #[cfg(feature = "rpm_filters")] 0.001,
    );

    // Initialize the IMU task context with the IMU provided by the Board Support Package.
    let imu_ctx = tasks::imu::init(hardware.imu);

    // Initialize the motor mixer task context with the motor driver provided by the Board Support Package.
    #[rustfmt::skip]
    let motor_mixer_ctx = tasks::motor_mixer::init(
        config.mixer,
        config.motor,
        hardware.motor_driver,
        #[cfg(feature = "rpm_filters")] config.rpm_notch_filter_bank,
        #[cfg(feature = "rpm_filters")] 0.001
    );

    let rx_ctx = tasks::rx::init(hardware.radio, config.rates);

    // TODO: Initialize the MSP task context with the UART provided by the Board Support Package.
    #[cfg(feature = "msp")]
    let msp_ctx = Some(tasks::msp::init());

    #[cfg(feature = "blackbox")]
    let blackbox_encoder_ctx = tasks::blackbox_encoder::init(config.blackbox);

    // TODO: Initialize the blackbox writer context with the storage provided by the Board Support Package.
    #[cfg(feature = "blackbox")]
    let blackbox_writer_ctx = Some(tasks::blackbox_writer::init());

    #[cfg(feature = "autopilot")]
    let autopilot_ctx = tasks::autopilot::init();

    #[cfg(feature = "barometer")]
    let barometer_ctx = hardware.barometer.map(tasks::barometer::init);

    // TODO: replace `Some` with `board.battery.map` or similar.
    #[cfg(feature = "battery")]
    let battery_ctx = Some(tasks::battery::init());

    #[cfg(feature = "gps")]
    let gps_ctx = hardware.gps.map(|gps| tasks::gps::init(gps.uart_rx, gps.uart_tx, gps.parser));

    #[cfg(feature = "magnetometer")]
    let magnetometer_ctx = hardware.magnetometer.map(tasks::magnetometer::init);

    #[cfg(feature = "optical_flow")]
    let optical_flow_ctx = hardware.optical_flow.map(tasks::optical_flow::init);

    #[cfg(feature = "osd")]
    let osd_ctx = Some(tasks::osd::init(display_port_mutex).await);

    #[cfg(feature = "rangefinder")]
    let rangefinder_ctx = hardware.rangefinder.map(tasks::rangefinder::init);

    // ==================================================
    // UnLock the GLOBAL_CONFIGs.
    // ==================================================

    drop(config);

    // ==================================================
    // Spawn the realtime tasks.
    // ==================================================

    #[rustfmt::skip]
    let realtime_spawner = {
        #[cfg(feature = "realtime_executor")] { crate::boards::start_realtime_executor() }
        #[cfg(not(feature = "realtime_executor"))] { spawner.make_send() }
    };

    // If the processor is multicore, then the gyro_pid task gets core1 all to itself.
    #[rustfmt::skip]
    let gyro_pid_spawner = {
        #[cfg(feature = "multicore")] { crate::boards::start_core1_executor() }
        #[cfg(not(feature = "multicore"))] { realtime_spawner }
    };

    #[allow(clippy::expect_used)]
    {
        gyro_pid_spawner.spawn(tasks::gyro_pid::run(gyro_pid_ctx).expect("Failed to create GYRO PID task"));
        // TODO: The IMU task is just used during development. It will at some point be removed.
        realtime_spawner.spawn(tasks::imu::run(imu_ctx).expect("Failed to create IMU task"));
        realtime_spawner.spawn(tasks::motor_mixer::run(motor_mixer_ctx).expect("Failed to create MOTOR MIXER task"));
        realtime_spawner.spawn(tasks::rx::run(rx_ctx).expect("Failed to create RX task"));
    }
    #[cfg(feature = "blackbox")]
    {
        // The blackbox_encoder runs on the realtime executor, the blackbox_writer runs on the background executor.
        if let Some(blackbox_writer_ctx) = blackbox_writer_ctx
            && let Ok(blackbox_encoder_task) = tasks::blackbox_encoder::run(blackbox_encoder_ctx)
            && let Ok(blackbox_writer_task) = tasks::blackbox_writer::run(blackbox_writer_ctx)
        {
            realtime_spawner.spawn(blackbox_encoder_task);
            spawner.spawn(blackbox_writer_task);
        }
    }

    // ==================================================
    // Spawn the background tasks.
    // ==================================================

    // Always try and spawn the Autopilot, since if we have any sensors at all enabled it can probably
    // perform some sort of assistance.
    #[cfg(feature = "autopilot")]
    if let Ok(autopilot_task) = tasks::autopilot::run(autopilot_ctx) {
        spawner.spawn(autopilot_task);
    }

    #[cfg(feature = "barometer")]
    if let Some(barometer_ctx) = barometer_ctx
        && let Ok(barometer_task) = tasks::barometer::run(barometer_ctx)
    {
        spawner.spawn(barometer_task);
    }

    #[cfg(feature = "battery")]
    if let Some(battery_ctx) = battery_ctx
        && let Ok(battery_task) = tasks::battery::run(battery_ctx)
    {
        spawner.spawn(battery_task);
    }

    #[cfg(feature = "gps")]
    if let Some(gps_ctx) = gps_ctx
        && let Ok(gps_task) = tasks::gps::run(gps_ctx)
    {
        spawner.spawn(gps_task);
    }

    #[cfg(feature = "magnetometer")]
    if let Some(magnetometer_ctx) = magnetometer_ctx
        && let Ok(magnetometer_task) = tasks::magnetometer::run(magnetometer_ctx)
    {
        spawner.spawn(magnetometer_task);
    }

    #[cfg(feature = "msp")]
    if let Some(msp_ctx) = msp_ctx
        && let Ok(msp_task) = tasks::msp::run(msp_ctx)
    {
        spawner.spawn(msp_task);
    }

    #[cfg(feature = "optical_flow")]
    if let Some(optical_flow_ctx) = optical_flow_ctx
        && let Ok(optical_flow_task) = tasks::optical_flow::run(optical_flow_ctx)
    {
        spawner.spawn(optical_flow_task);
    }

    #[cfg(feature = "osd")]
    if let Some(osd_ctx) = osd_ctx
        && let Ok(osd_task) = tasks::osd::run(osd_ctx)
    {
        spawner.spawn(osd_task);
    }

    #[cfg(feature = "rangefinder")]
    if let Some(rangefinder_ctx) = rangefinder_ctx
        && let Ok(rangefinder_task) = tasks::rangefinder::run(rangefinder_ctx)
    {
        spawner.spawn(rangefinder_task);
    }
}
