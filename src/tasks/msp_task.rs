#![allow(unused)]
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
/// MSP Placeholder.
///
use embassy_time::Duration;

use log::info;
use static_cell::StaticCell;
use stream_buf::{StreamBufReader, StreamBufWriter};

use crate::{
    config::{CONFIG_PUB_SUB_CHANNEL, GYRO_PID_PUB_SUB_CHANNEL},
    multiwii_serial_protocol::Msp,
};

pub(crate) static MSP_CTX: StaticCell<MspContext> = StaticCell::new();

/// Context for MSP task.
///
pub struct MspContext {
    pub msp: Msp,
    pub read_buf: [u8; Self::READ_BUF_SIZE],
    pub write_buf: [u8; Self::WRITE_BUF_SIZE],
}

impl MspContext {
    pub const READ_BUF_SIZE: usize = 256;
    pub const WRITE_BUF_SIZE: usize = 512;

    /// Helper to get a reader for read_buf.
    pub fn reader(&mut self) -> StreamBufReader<'_> {
        StreamBufReader::new(&self.read_buf)
    }

    /// Helper to get a writer for write_buf.
    pub fn writer(&mut self) -> StreamBufWriter<'_> {
        StreamBufWriter::new(&mut self.write_buf)
    }
}

#[embassy_executor::task]
pub async fn msp_task(ctx: &'static mut MspContext) {
    let config_publisher = CONFIG_PUB_SUB_CHANNEL.publisher().expect("failed to create MSP config publisher");
    let gyro_pid_publisher = GYRO_PID_PUB_SUB_CHANNEL.publisher().expect("failed to create MSP gyro_pid publisher");
    // for now just wait on a ticker to drive the MSP loop. TODO: change this to wait on an MSP packet instead.
    let mut ticker = embassy_time::Ticker::every(Duration::from_millis(200));
    let mut loop_count: u32 = 0;

    info!("     MSP: task started");
    loop {
        // Wait for msp packet
        // let msp_packet = msp.receive().await;
        ticker.next().await; // for now just wait on ticker

        // Generally, we don't want to store the Reader itself because it tracks a "cursor" (current position).
        // It's better to store the data and create a fresh reader whenever we start processing a new packet.
        let mut src = StreamBufReader::new(&ctx.read_buf);

        let cmd_msp = Msp::SET_FAILSAFE_CONFIG;
        let _result = Msp::process_read_command(cmd_msp, &mut src, &config_publisher, &gyro_pid_publisher).await;

        info!("     MSP:      loop {loop_count}");
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
