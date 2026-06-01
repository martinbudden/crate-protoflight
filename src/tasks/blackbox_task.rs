#![allow(unused)]
use blackbox_logger::{
    Blackbox, Event, SetpointMessage, SliceWriter, StateMachine, drivers::SdStorage, sd_card::MockSdCard,
};
use log::info;

use crate::tasks::dispatch::{GyroPidReceiver, SetpointReceiver};

pub(crate) static BLACKBOX_CTX: static_cell::StaticCell<BlackboxContext> = static_cell::StaticCell::new();

pub struct BlackboxContext {
    pub gyro_pid_receiver: GyroPidReceiver,
    pub setpoint_receiver: SetpointReceiver,
    pub setpoint_message: SetpointMessage,
    pub blackbox: Blackbox,
    pub sd_card: MockSdCard,
    pub buffer: [u8; 1024],
    pub pos: usize,
    //pub slice_writer: SliceWriter<'static>,
}

impl BlackboxContext {
    // We take the buffer as a mutable reference to the array
    pub fn slice_writer(buffer: &mut [u8; 1024], pos: usize) -> SliceWriter<'_> {
        SliceWriter {
            // Rust automatically coerces &mut [u8; 1024] to &mut [u8]
            buffer,
            pos,
        }
    }
}

/// Blackbox task placeholder.
#[embassy_executor::task]
pub async fn blackbox_task(ctx: &'static mut BlackboxContext) {
    info!(" BLACKBOX: task started");
    let mut time_us: u32 = 0;
    let mut loop_count: u32 = 0;

    ctx.blackbox.set_state(StateMachine::LogFileHeader);

    // write the Blackbox header.
    loop {
        let len = {
            let mut slice_writer = BlackboxContext::slice_writer(&mut ctx.buffer, ctx.pos);
            ctx.blackbox.update(&mut slice_writer, time_us)
        };
        _ = ctx.sd_card.write_all(&ctx.buffer[..len]).await;
        info!("BLACKBOX: loop {loop_count}");
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
        if ctx.blackbox.state() == StateMachine::Running {
            break;
        }
    }

    loop_count = 0;
    let mut index = 0;
    loop {
        time_us = time_us.wrapping_add(125);
        let gyro_pid_msg = ctx.gyro_pid_receiver.changed().await; // blocking
        // non-blocking
        if let Some(setpoint_msg) = ctx.setpoint_receiver.try_get() {
            ctx.setpoint_message = setpoint_msg;
        }
        ctx.blackbox.load_telemetry(time_us, gyro_pid_msg, ctx.setpoint_message);
        let len = {
            let mut slice_writer = BlackboxContext::slice_writer(&mut ctx.buffer, ctx.pos);
            ctx.blackbox.update(&mut slice_writer, time_us)
        };
        if index == 512 {
            // write End of log
            let len = {
                let mut slice_writer = BlackboxContext::slice_writer(&mut ctx.buffer, ctx.pos);
                ctx.blackbox.logger.log_e_frame(&mut slice_writer, Event::LogEnd)
            };
            ctx.sd_card.write_all(&ctx.buffer[..len]).await;
            info!("**** BLACKBOX: END OF LOG");
        }
        _ = ctx.sd_card.write_all(&ctx.buffer[..len]).await;
        if loop_count.is_multiple_of(10) {
            info!(" BLACKBOX: loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
        index += 1;
    }

    /*loop {
        // Wait until the sender updates the value.
        // The primary client of the ahrs_receiver is the motor_mixer
        let gyro_pid_msg = ctx.gyro_pid_receiver.changed().await;
        ctx.blackbox.load_telemetry(time_us, gyro_pid_msg, setpoint_msg);

        info!("BLACKBOX: Received time_us {}", gyro_pid_msg.time_us);
        let len = 0;
        info!(
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
