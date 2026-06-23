#![cfg(feature = "blackbox")]

use blackbox_logger::{Blackbox, BlackboxEvent, BlackboxMainData, BlackboxSlowData, LoggerState, SliceEncoder};

#[cfg(feature = "gps")]
use blackbox_logger::{BlackboxGpsData, BlackboxGpsPosition};

use crate::{
    drivers::sd_card::{MockSdCard, SdStorage},
    sensors::{GyroPidMessage, SetpointMessage},
    tasks::gyro_pid_task::{GyroPidReceiver, SetpointReceiver},
};

#[cfg(feature = "gps")]
use crate::{
    gps::{GpsMessage, GpsSolutionData},
    tasks::gps_task::GpsSubscriber,
};

pub struct BlackboxContext<'a> {
    pub gyro_pid_receiver: GyroPidReceiver,
    pub setpoint_receiver: SetpointReceiver,
    pub setpoint_message: SetpointMessage,
    #[cfg(feature = "gps")]
    pub gps_subscriber: GpsSubscriber<'a>,
    pub blackbox: Blackbox,
    pub sd_card: MockSdCard,
    pub buffer: [u8; 1024],
    pub pos: usize,
    //pub slice_writer: SliceEncoder<'static>,
}

impl BlackboxContext<'_> {
    // We take the buffer as a mutable reference to the array
    pub fn slice_writer(buffer: &mut [u8; 1024], pos: usize) -> SliceEncoder<'_> {
        SliceEncoder {
            // Rust automatically coerces &mut [u8; 1024] to &mut [u8]
            buffer,
            pos,
        }
    }
}

/// Blackbox task placeholder.
#[embassy_executor::task]
pub async fn blackbox_task(ctx: &'static mut BlackboxContext<'static>) {
    log::info!(" BLACKBOX: task started");
    let mut time_us: u32 = 0;
    let mut loop_count: u32 = 0;

    // write the Blackbox log file header.
    ctx.blackbox.set_state(LoggerState::WriteFileHeader);
    while ctx.blackbox.state() != LoggerState::HeaderWritten {
        let len = {
            let mut slice_writer = BlackboxContext::slice_writer(&mut ctx.buffer, ctx.pos);
            ctx.blackbox.update(&mut slice_writer, time_us, true)
        };
        _ = ctx.sd_card.write_all(&ctx.buffer[..len]).await;
        log::info!("BLACKBOX:  hdr {loop_count},{len}");
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }

    loop_count = 0;
    let mut index = 0;
    loop {
        time_us = time_us.wrapping_add(125);
        let gyro_pid_msg = ctx.gyro_pid_receiver.changed().await; // blocking
        // non-blocking
        if let Some(setpoint_msg) = ctx.setpoint_receiver.try_get() {
            ctx.setpoint_message = setpoint_msg;
            let slow_data = slow_data_from(ctx.setpoint_message);
            ctx.blackbox.set_slow_data(slow_data);
        }
        let main_data = main_data_from(time_us, gyro_pid_msg, ctx.setpoint_message);
        ctx.blackbox.set_main_data(time_us, main_data);

        #[cfg(feature = "gps")]
        if let Some(wait_result) = ctx.gps_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(event) = wait_result
            && let GpsMessage::GpsSolution(gps_solution_data) = event
        {
            let gps_data = gps_data_from(gps_solution_data);
            ctx.blackbox.set_gps_data(gps_data);
        }

        let len = {
            let mut slice_writer = BlackboxContext::slice_writer(&mut ctx.buffer, ctx.pos);
            ctx.blackbox.update(&mut slice_writer, time_us, true)
        };
        if index == 512 {
            // write End of log
            let len = {
                let mut slice_writer = BlackboxContext::slice_writer(&mut ctx.buffer, ctx.pos);
                ctx.blackbox.logger.log_e_frame(&mut slice_writer, BlackboxEvent::LogEnd);
                slice_writer.pos
            };
            _ = ctx.sd_card.write_all(&ctx.buffer[..len]).await;
            log::info!("**** BLACKBOX: END OF LOG");
        }
        _ = ctx.sd_card.write_all(&ctx.buffer[..len]).await;
        if loop_count.is_multiple_of(10) {
            log::info!("      BLACKBOX: loop {loop_count},{len}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
        index += 1;
    }

    /*loop {
        // Wait until the sender updates the value.
        // The primary client of the ahrs_receiver is the motor_mixer
        let gyro_pid_msg = ctx.gyro_pid_receiver.changed().await;
        ctx.blackbox.load_telemetry(time_us, gyro_pid_msg, setpoint_msg);

        log::info!("BLACKBOX: Received time_us {}", gyro_pid_msg.time_us);
        let len = 0;
        log::info!(
            "BLACKBOX: Encoded frame in {} bytes. (x: {}, y: {}, z: {}),(x: {}, y: {}, z: {})",
            len,
            gyro_pid_msg.gyro_rps.x,
            gyro_pid_msg.gyro_rps.y,
            gyro_pid_msg.gyro_rps.z,
            gyro_pid_msg.gyro_rps_unfiltered.x,
            gyro_pid_msg.gyro_rps_unfiltered.y,
            gyro_pid_msg.gyro_rps_unfiltered.z
        );
        // Process logging.
        /*let buf = [b'a', b'b', b'c', b'd', b'e', b'f'];
        let len = 6;
        ctx.sd_card.write_all(&buf[..len]).await;*/
        // 3. Increment fake time (e.g., 1000us per sample for 1kHz)
        time_us = time_us.wrapping_add(1000); // use wrapping_add to handle when time rolls over at max u32.
    }*/
}

#[inline]
pub fn main_data_from(
    _current_time_us: u32,
    gyro_pid_msg: GyroPidMessage,
    setpoint_msg: SetpointMessage,
) -> BlackboxMainData {
    const TO_I16: f32 = 32_757.0;
    let motor_commands = gyro_pid_msg.motor_commands * 2.0;
    BlackboxMainData {
        time_us: gyro_pid_msg.time_us,
        baro_altitude: 0,
        range_raw: 0,
        amperage: 0,
        battery_voltage: 0,
        rssi: 0,
        // todo, add scaling to below
        #[allow(clippy::cast_possible_truncation)]
        pid_p: gyro_pid_msg.pid_errors_p.map(|x| x as i32),
        #[allow(clippy::cast_possible_truncation)]
        pid_i: gyro_pid_msg.pid_errors_i.map(|x| x as i32),
        #[allow(clippy::cast_possible_truncation)]
        pid_d: [gyro_pid_msg.pid_errors_d[0] as i32, gyro_pid_msg.pid_errors_d[1] as i32, 0],
        pid_s: [0i32; BlackboxMainData::RPY_AXIS_COUNT],
        pid_k: [0i32; BlackboxMainData::RPY_AXIS_COUNT],

        rc_commands: [1500, 1500, 1500, 1100],

        // TODO: need to scale these
        #[allow(clippy::cast_possible_truncation)]
        setpoints: [motor_commands.x as i16, motor_commands.y as i16, motor_commands.z as i16, motor_commands.t as i16],
        gyro: (gyro_pid_msg.gyro_rps.to_degrees()).into(),
        gyro_unfiltered: (gyro_pid_msg.gyro_rps_unfiltered.to_degrees()).into(),
        acc: (gyro_pid_msg.acc * 4096.0).into(),
        #[cfg(feature = "magnetometer")]
        mag: [0i16; BlackboxMainData::XYZ_AXIS_COUNT],

        #[allow(clippy::cast_possible_truncation)]
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
        #[cfg(feature = "eight_motors")]
        motor: [1100, 1100, 1100, 1100, 1100, 1100, 1100, 1100],
        #[cfg(not(feature = "eight_motors"))]
        motor: [1100, 1100, 1100, 1100],
        #[cfg(feature = "dshot_telemetry")]
        erpm: [0u16; BlackboxMainData::MAX_SUPPORTED_MOTOR_COUNT],

        debug: [
            gyro_pid_msg.debug[0],
            gyro_pid_msg.debug[1],
            gyro_pid_msg.debug[2],
            gyro_pid_msg.debug[3],
            gyro_pid_msg.debug[4],
            gyro_pid_msg.debug[5],
            setpoint_msg.debug[0],
            setpoint_msg.debug[1],
        ],
        #[cfg(feature = "servos")]
        servos: [0i16; MainData::MAX_SUPPORTED_SERVO_COUNT],
    }
}

#[inline]
pub fn slow_data_from(setpoint: SetpointMessage) -> BlackboxSlowData {
    BlackboxSlowData {
        flight_mode_flags: setpoint.flight_mode_flags,
        state_flags: setpoint.state_flags,
        failsafe_phase: setpoint.failsafe_phase,
        rx_signal_received: setpoint.rx_signal_received,
        rx_flight_channel_is_valid: setpoint.rx_flight_channel_is_valid,
    }
}

#[cfg(feature = "gps")]
#[inline]
pub fn gps_data_from(gps: GpsSolutionData) -> BlackboxGpsData {
    BlackboxGpsData {
        time_of_week_ms: gps.time,
        interval_ms: 0,
        position: BlackboxGpsPosition {
            longitude_degrees_1e7: gps.llh.longitude_degrees_x1e7,
            latitude_degrees_1e7: gps.llh.latitude_degrees_x1e7,
            altitude_cm: gps.llh.altitude_cm,
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
