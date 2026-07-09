use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

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

#[cfg(feature = "rp2350")]
use crate::tasks::init_rp;

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

#[cfg(all(feature = "blackbox", feature = "rp2350"))]
use crate::tasks::sd_writer_task::{SdWriterContext, sd_writer_task};

#[cfg(all(feature = "blackbox", feature = "std"))]
use crate::drivers::sd_card::MockSdCard;

#[cfg(feature = "blackbox")]
use {
    crate::tasks::blackbox_task::{BlackboxContext, blackbox_task},
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

    #[cfg(feature = "rp2350")]
    let (_gyro_res, _gyro_interrupt, _blackbox_res, _aux_pio_res, _uart0, _uart1, _i2c0, flash) = init_rp::init_rp();

    // --- INITIALIZE MOCK STUB (HOST PROFILE ENVIRONMENT) ---
    #[allow(unused)]
    #[cfg(not(feature = "max7456"))]
    let display_ref = { DISPLAY_PORT_MUTEX_CELL.init(Mutex::new(DisplayPortMock::default())) };
    #[cfg(all(feature = "serde", feature = "rp2350"))]
    load_global_configs(flash).await;
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
            #[cfg(all(feature = "blackbox", feature = "std"))]
            sd_card: MockSdCard::new("blackbox_log.bbl"),
        })
    };
    #[cfg(all(feature = "blackbox", feature = "rp2350"))]
    let sd_writer_ctx = {
        //if let Ok(blackbox_spi) = _blackbox_res {
        //    SD_WRITER_CTX.init(SdWriterContext::new(blackbox_spi))
        //}
        SD_WRITER_CTX.init(SdWriterContext::new(_blackbox_res.unwrap()))
    };

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
