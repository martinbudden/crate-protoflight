#![cfg(feature = "blackbox")]
#![allow(unused)]

use blackbox_logger::{
    Blackbox, BlackboxConfig, BlackboxDateTime, BlackboxMainData, BlackboxSlowData, BlackboxSysInfo, FieldSelect,
    LoggerState, SliceEncoder,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use radio_controllers::RcMode;
use static_cell::StaticCell;

#[cfg(feature = "gps")]
use crate::tasks::gps::gps_subscriber;
use crate::{
    sensors::{GyroPidMessage, SetpointMessage},
    tasks::gyro_pid::{GyroPidReceiver, SetpointReceiver, gyro_pid_receiver, setpoint_receiver},
};

#[cfg(feature = "barometer")]
use crate::tasks::barometer::{BarometerSubscriber, barometer_subscriber};

#[cfg(feature = "battery")]
use crate::tasks::battery::{BatterySubscriber, battery_subscriber};

#[cfg(feature = "debug")]
use crate::tasks::{DebugMode, GLOBAL_DEBUG};

#[cfg(feature = "gps")]
use {
    crate::{
        gps::{GpsMessage, GpsSolutionData},
        tasks::gps::GpsSubscriber,
    },
    blackbox_logger::{BlackboxGpsData, BlackboxGpsPosition},
};

static BLACKBOX_ENCODER_CTX: StaticCell<BlackboxEncoderContext> = StaticCell::new();

#[allow(unused)]
#[rustfmt::skip]
pub struct BlackboxEncoderContext {
    pub gyro_pid_receiver: GyroPidReceiver,
    pub setpoint_receiver: SetpointReceiver,
    pub setpoint_message: SetpointMessage,
    pub barometer_altitude: i32,
    pub battery_current: i16,
    pub battery_voltage: u16,
    pub range_raw: i32,
    pub rssi: u16,
    #[cfg(feature = "barometer")] pub barometer_subscriber: BarometerSubscriber,
    #[cfg(feature = "battery")] pub battery_subscriber: BatterySubscriber,
    #[cfg(feature = "gps")] pub gps_subscriber: GpsSubscriber,
    pub blackbox: Blackbox,
    pub buffer: [u8; BlackboxEncoderContext::BUFFER_CAPACITY],
    pub overflow_counter: u32,
}

impl BlackboxEncoderContext {
    const BUFFER_CAPACITY: usize = 1024;

    #[rustfmt::skip]
    pub fn new(config: BlackboxConfig) -> Self {
        //let mut blackbox_config = blackbox_config;
        //blackbox_config.huffman_compress = true;

        //nvs::load_blackbox_config(&mut config.blackbox, &mut flash_driver, config_flash_range.clone());
        let mut config = config;
        config.fields_disabled_mask = FieldSelect::PID_STERM_ROLL
        | FieldSelect::PID_STERM_PITCH
        | FieldSelect::PID_STERM_YAW
        | FieldSelect::PID_KTERM
        //| FieldSelect::PID
        | FieldSelect::RSSI
        //| FieldSelect::SETPOINT
        //| FieldSelect::GYRO_UNFILTERED
        //| FieldSelect::MOTOR_RPM
        | FieldSelect::BATTERY_VOLTAGE
        | FieldSelect::BATTERY_CURRENT
        | FieldSelect::BAROMETER
        | FieldSelect::RANGEFINDER
        | FieldSelect::ATTITUDE
        //| FieldSelect::ACCELEROMETER
        //| FieldSelect::GYRO
        //| FieldSelect::RC_COMMANDS
        //| FieldSelect::MOTOR
        | FieldSelect::MAGNETOMETER;

        // TODO: derive blackbox sys info from config.
        let sys_info = BlackboxSysInfo {
            features: 541_130_760,
            gyro_scale: 0x3f80_0000,
            looptime: 125, // 125us = 8kHz gyro/pid loop
            gyro_sync_denom: 1,
            pid_process_denom: 1,
            acc_1g: 4096,
            motor_output_min: 48,
            motor_output_max: 2047,
            vbat_scale: 0,
            vbat_min_cell_voltage: 330,
            vbat_warning_cell_voltage: 350,
            vbat_max_cell_voltage: 430,
            current_sensor_scale: 0,
            current_sensor_offset: 250,
            date_time: BlackboxDateTime::new(),
            motor_pole_count: 14,
        };

        // NRVO (Named Return Value Optimization) ensures blackbox is created in place and not copied.
        let mut blackbox = Blackbox::new(config, sys_info);
        blackbox.init();

        Self {
            gyro_pid_receiver: gyro_pid_receiver(),
            setpoint_receiver: setpoint_receiver(),
            setpoint_message: SetpointMessage::new(),
            barometer_altitude: 0,
            battery_current: 0,
            battery_voltage: 0,
            range_raw: 0,
            rssi: 0,
            #[cfg(feature = "barometer")] barometer_subscriber: barometer_subscriber(),
            #[cfg(feature = "battery")] battery_subscriber: battery_subscriber(),
            #[cfg(feature = "gps")] gps_subscriber: gps_subscriber(),
            blackbox,
            buffer: [0u8; Self::BUFFER_CAPACITY],
            overflow_counter: 0,
        }
    }
}

/// A fixed-size message container used to send blackbox chunks from this task to the `blackbox_writer` task.
/// Set the internal buffer capacity to a size larger than the maximum possible `len` frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlackboxWriteBlock {
    pub data: [u8; Self::CAPACITY], // Adjust size to match the largest expected serialized packet length
    pub len: usize,
}

impl Default for BlackboxWriteBlock {
    fn default() -> Self {
        Self::new(0)
    }
}

impl BlackboxWriteBlock {
    pub const CAPACITY: usize = 64;

    #[inline]
    pub const fn new(len: usize) -> Self {
        Self { data: [0u8; Self::CAPACITY], len }
    }

    #[inline]
    pub fn from_chunk(slice: &[u8]) -> Self {
        // Enforce the size boundary strictly using compile-time constants
        let copy_len = slice.len().min(Self::CAPACITY);
        let mut block = Self { data: [0u8; Self::CAPACITY], len: copy_len };
        block.data[..copy_len].copy_from_slice(&slice[..copy_len]);
        block
    }

    #[inline]
    pub fn send_data_to_blackbox_writer_task(data: &[u8], overflow_counter: &mut u32) -> bool {
        let mut ret = false;
        let overflow_counter_in = *overflow_counter;
        // Loop through the slice in chunks matching BlackboxWriteBlock capacity
        for chunk in data.chunks(Self::CAPACITY) {
            let block = Self::from_chunk(chunk);
            let block_item = BlackboxWriteItem::Data(block);

            // We use non-blocking `try_send` here.
            // So if the `BLACKBOX_WRITE_QUEUE` is full the task won't stall,
            // Instead the chunk is just dropped.
            if let Err(_overflow) = BLACKBOX_WRITE_QUEUE.try_send(block_item) {
                ret = true;
                *overflow_counter = overflow_counter.wrapping_add(1);
                log::error!("BLACKBOX: FIFO queue full! Dropped a logging chunk.");
            }
        }
        ret
    }
}

pub enum BlackboxWriteItem {
    Data(BlackboxWriteBlock),
    Flush,
}

const BLACKBOX_WRITE_QUEUE_COUNT: usize = 256;
pub static BLACKBOX_WRITE_QUEUE: Channel<CriticalSectionRawMutex, BlackboxWriteItem, BLACKBOX_WRITE_QUEUE_COUNT> =
    Channel::new();

pub fn init(config: BlackboxConfig) -> &'static mut BlackboxEncoderContext {
    BLACKBOX_ENCODER_CTX.init(BlackboxEncoderContext::new(config))
}

/// Blackbox encoder task.
///
/// REALTIME PRIORITY                         BACKGROUND PRIORITY
///
/// blackbox encoder task
///    │
///    │ produces ~40 bytes
///    ▼
/// `BlackboxWriteBlock`
///    │
///    │ copy/move into Channel
///    ▼
/// ┌─────────────────────────────────────────────┐
/// │ Embassy Channel:                            │
/// | 256 × `BlackboxWriteBlock`                  │
/// │                                             │
/// │ [40] [40] [40] [40] [40] ...                │
/// │                                             │
/// └──────────────────────────────────────────┬──┘
///                                            │
///                                            ▼
///                                        blackbox writer task
///                                            │
///                                            ▼
///                                        batch records
///                                            │
///                                            ▼
///                                        512-byte buffer
///                                            │
///                                            ▼
///                                         SD card
#[embassy_executor::task]
pub async fn run(ctx: &'static mut BlackboxEncoderContext) {
    log::info!("    BLACKBOX: task started");
    let mut loop_count: u32 = 0;

    // Write the Blackbox log file header by using `blackbox.update` to step through the blackbox state machine
    // until the state is `LoggerState::HeaderWritten`.
    #[cfg(feature = "debug")]
    ctx.blackbox.start(u16::from(GLOBAL_DEBUG.mode()));
    #[cfg(not(feature = "debug"))]
    ctx.blackbox.start(0);
    while ctx.blackbox.state() != LoggerState::HeaderWritten {
        let time_us = 0;
        let len = ctx.blackbox.update(&mut SliceEncoder::new(&mut ctx.buffer), time_us, false);
        _ = BlackboxWriteBlock::send_data_to_blackbox_writer_task(&ctx.buffer[..len], &mut ctx.overflow_counter);
        //log::info!("BLACKBOX:  hdr {loop_count},{len}");
        loop_count = loop_count.wrapping_add(1);
    }
    log::info!("    BLACKBOX: header written {loop_count}");

    loop_count = 0;
    let mut force_i_frame = false;
    loop {
        // blocking
        let gyro_pid_msg = ctx.gyro_pid_receiver.changed().await;
        let time_us = gyro_pid_msg.time_us;
        // non-blocking
        if let Some(setpoint_message) = ctx.setpoint_receiver.try_get() {
            // if we have a new `setpoint_message` then update `ctx.setpoint_message` so that the most up to date `setpoint_message` is used.
            ctx.setpoint_message = setpoint_message;
            let mut slow_data = slow_data_from(ctx.setpoint_message);
            if setpoint_message.rc_modes.test(RcMode::ARM) {
                slow_data.set_blackbox_active(true);
            }
            ctx.blackbox.set_slow_data(slow_data);
        }

        #[cfg(feature = "barometer")]
        if let Some(wait_result) = ctx.barometer_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(event) = wait_result
            && let barometer_message = event
        {
            ctx.barometer_altitude = barometer_message.altitude_m_i32;
        }

        #[allow(clippy::cast_possible_truncation)]
        #[cfg(feature = "battery")]
        if let Some(wait_result) = ctx.battery_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(event) = wait_result
            && let battery_message = event
        {
            ctx.battery_voltage = battery_message.voltage.unfiltered_x100;
            ctx.battery_current = battery_message.current.amperage_latest_x100 as i16;
        }

        // `set_main_data` always uses the most up to date `setpoint_message`.
        ctx.blackbox.set_main_data(main_data_from(
            gyro_pid_msg,
            ctx.setpoint_message,
            ctx.barometer_altitude,
            ctx.battery_current,
            ctx.battery_voltage,
            ctx.range_raw,
            ctx.rssi,
        ));

        #[cfg(feature = "gps")]
        if let Some(wait_result) = ctx.gps_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(event) = wait_result
            && let GpsMessage::Solution(gps_solution_data) = event
        {
            let gps_data = gps_data_from(gps_solution_data);
            ctx.blackbox.set_gps_data(gps_data);
        }

        let len = ctx.blackbox.update(&mut SliceEncoder::new(&mut ctx.buffer), time_us, force_i_frame);
        // Force the blackbox logger to write an I Frame (keyframe) the next time around if there was an overflow.
        force_i_frame =
            BlackboxWriteBlock::send_data_to_blackbox_writer_task(&ctx.buffer[..len], &mut ctx.overflow_counter);

        if ctx.blackbox.is_active() && loop_count.is_multiple_of(10) {
            log::info!("BLACKBOX: loop {loop_count},{len},{0}", ctx.overflow_counter);
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}

#[inline]
pub fn main_data_from(
    gyro_pid_msg: GyroPidMessage,
    setpoint_message: SetpointMessage,
    barometer_altitude: i32,
    battery_current: i16,
    battery_voltage: u16,
    range_raw: i32,
    rssi: u16,
) -> BlackboxMainData {
    const TO_I16: f32 = 32_757.0;

    // let motor_commands = gyro_pid_msg.motor_commands * 2.0;
    let setpoints = setpoint_message.setpoints;
    let k = 1000.0f32;

    #[allow(clippy::cast_possible_truncation)]
    BlackboxMainData {
        time_us: gyro_pid_msg.time_us,
        barometer_altitude,
        battery_current,
        battery_voltage,
        range_raw,
        rssi,
        // TODO: add scaling to below
        pid_p: gyro_pid_msg.pid_errors_p.map(|x| x as i32),
        pid_i: gyro_pid_msg.pid_errors_i.map(|x| x as i32),
        pid_d: [gyro_pid_msg.pid_errors_d[0] as i32, gyro_pid_msg.pid_errors_d[1] as i32, 0],
        pid_s: setpoint_message.pid_errors_s.map(|x| x as i32),
        pid_k: setpoint_message.pid_errors_k.map(|x| x as i32),

        rc_commands: setpoint_message.rc_commands,

        // TODO: need to scale these
        setpoints: [
            (k * setpoints[0]) as i16,
            (k * setpoints[1]) as i16,
            (k * setpoints[2]) as i16,
            (k * setpoints[3]) as i16,
        ],
        gyro: (gyro_pid_msg.gyro_rps.to_degrees()).into(),
        gyro_unfiltered: (gyro_pid_msg.gyro_rps_unfiltered.to_degrees()).into(),
        acc: (gyro_pid_msg.acc * 4096.0).into(),
        #[cfg(feature = "magnetometer")]
        mag: [0i16; BlackboxMainData::XYZ_AXIS_COUNT],

        orientation: if gyro_pid_msg.orientation.w > 0.0 {
            [
                (gyro_pid_msg.orientation.x * TO_I16) as i16,
                (gyro_pid_msg.orientation.y * TO_I16) as i16,
                (gyro_pid_msg.orientation.z * TO_I16) as i16,
            ]
        } else {
            [
                (-gyro_pid_msg.orientation.x * TO_I16) as i16,
                (-gyro_pid_msg.orientation.y * TO_I16) as i16,
                (-gyro_pid_msg.orientation.z * TO_I16) as i16,
            ]
        },

        motor: [1100i16; BlackboxMainData::MAX_SUPPORTED_MOTOR_COUNT],
        #[cfg(feature = "dshot_telemetry")]
        erpm_d2: setpoint_message.motor_rpm_d2,

        #[cfg(feature = "debug")]
        debug: crate::tasks::GLOBAL_DEBUG.values(),

        #[cfg(feature = "servos")]
        servos: setpoint_message.servos,
    }
}

#[inline]
pub fn slow_data_from(setpoint_message: SetpointMessage) -> BlackboxSlowData {
    BlackboxSlowData {
        flight_mode_flags: setpoint_message.rc_modes.bits_0_31(),
        gps_state_flags: setpoint_message.gps_state_flags,
        failsafe_phase: setpoint_message.failsafe_phase,
        rx_signal_received: setpoint_message.rx_signal_received,
        rx_flight_channel_is_valid: setpoint_message.rx_flight_channel_is_valid,
    }
}

#[cfg(feature = "gps")]
#[inline]
pub fn gps_data_from(gps: GpsSolutionData) -> BlackboxGpsData {
    BlackboxGpsData {
        time_of_week_ms: gps.time,
        interval_ms: 0,
        position: BlackboxGpsPosition {
            longitude_degrees_x1e7: gps.longitude_degrees_x1e7,
            latitude_degrees_x1e7: gps.latitude_degrees_x1e7,
            altitude_cm: gps.altitude_cm,
        },
        velocity_north_cmps: gps.velocity_ned_cmps.north,
        velocity_east_cmps: gps.velocity_ned_cmps.east,
        velocity_down_cmps: gps.velocity_ned_cmps.down,
        speed3d_cmps: gps.speed3d_cmps.cast_signed(),
        ground_speed_cmps: gps.ground_speed_cmps.cast_signed(),
        ground_course_degrees_x10: gps.ground_course_degrees_x10.cast_signed(),
        satellite_count: gps.satellite_count,
    }
}
