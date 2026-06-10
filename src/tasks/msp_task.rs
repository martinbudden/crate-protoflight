#![cfg(feature = "msp")]
#![allow(unused)]

//use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use log::info;
use stream_buf::{StreamBufReader, StreamBufWriter};

use crate::{
    config::{ConfigPublisher, FastConfigPublisher},
    multiwii_serial_protocol::{Msp, MspSensorData},
};

#[cfg(feature = "barometer")]
use crate::tasks::barometer_task::BarometerSubscriber;

#[cfg(feature = "battery")]
use crate::tasks::battery_task::BatterySubscriber;

#[cfg(feature = "gps")]
use crate::{gps::GpsMessage, tasks::gps_task::GpsSubscriber};

#[cfg(feature = "optical_flow")]
use crate::tasks::optical_flow_task::OpticalFlowSubscriber;

#[cfg(feature = "rangefinder")]
use crate::tasks::rangefinder_task::RangefinderSubscriber;

/// Context for MSP task.
///
pub const MSP_READ_BUF_SIZE: usize = 256;
pub const MSP_WRITE_BUF_SIZE: usize = 512;
pub struct MspContext<'a> {
    pub fast_config_publisher: FastConfigPublisher<'a>,
    pub config_publisher: ConfigPublisher<'a>,
    #[cfg(feature = "barometer")]
    pub barometer_subscriber: BarometerSubscriber<'a>,
    #[cfg(feature = "battery")]
    pub battery_subscriber: BatterySubscriber<'a>,
    #[cfg(feature = "gps")]
    pub gps_subscriber: GpsSubscriber<'a>,
    #[cfg(feature = "optical_flow")]
    pub optical_flow_subscriber: OpticalFlowSubscriber<'a>,
    #[cfg(feature = "rangefinder")]
    pub rangefinder_subscriber: RangefinderSubscriber<'a>,
    pub msp: Msp,
    pub read_buf: [u8; MSP_READ_BUF_SIZE],
    pub write_buf: [u8; MSP_WRITE_BUF_SIZE],
}

impl MspContext<'_> {
    /// Helper to get a reader for `read_buf`.
    pub fn reader(&'_ mut self) -> StreamBufReader<'_> {
        StreamBufReader::new(&self.read_buf)
    }

    /// Helper to get a writer for `write_buf`.
    pub fn writer(&'_ mut self) -> StreamBufWriter<'_> {
        StreamBufWriter::new(&mut self.write_buf)
    }
}

/// MSP task Placeholder.
#[embassy_executor::task]
pub async fn msp_task(ctx: &'static mut MspContext<'static>) {
    // for now just wait on a ticker to drive the MSP loop. TODO: change this to wait on an MSP packet instead.
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(200));
    let mut loop_count: u32 = 0;

    // value to pass to Msp::process_write_command
    let mut msp_sensor_data = MspSensorData::new();

    info!("      MSP: task started");
    loop {
        // Wait for msp packet
        // let msp_packet = msp.receive().await;
        ticker.next().await; // for now just wait on ticker

        #[cfg(feature = "barometer")]
        #[allow(clippy::cast_possible_truncation)]
        if let Some(wait_result) = ctx.barometer_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(barometer_data) = wait_result
        {
            msp_sensor_data.barometer_altitude_cm = ((barometer_data.altitude_m * 100.0) as i32).cast_unsigned();
        }

        #[cfg(feature = "rangefinder")]
        #[allow(clippy::cast_possible_truncation)]
        if let Some(wait_result) = ctx.rangefinder_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(rangefinder_message) = wait_result
        {
            msp_sensor_data.rangefinder_altitude_cm = ((rangefinder_message.distance_m * 100.0) as i32).cast_unsigned();
        }

        #[cfg(feature = "gps")]
        if let Some(wait_result) = ctx.gps_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(event) = wait_result
            && let GpsMessage::GpsSolution(gps_solution_data) = event
        {
            msp_sensor_data.gps_sol.llh = gps_solution_data.llh;
            msp_sensor_data.gps_sol.satellite_count = gps_solution_data.satellite_count;
            msp_sensor_data.gps_sol.ground_speed_cmps = gps_solution_data.ground_speed_cmps;
            msp_sensor_data.gps_sol.ground_course_degrees_x10 = gps_solution_data.ground_course_degrees_x10;
            msp_sensor_data.gps_sol.dop_positional = gps_solution_data.dop.positional;
        }

        // Generally, we don't want to store the Reader itself because it tracks a "cursor" (current position).
        // It's better to store the data and create a fresh reader whenever we start processing a new packet.
        let mut src = StreamBufReader::new(&ctx.read_buf);

        let cmd_msp = Msp::SET_FAILSAFE_CONFIG;
        let _result =
            Msp::process_read_command(cmd_msp, &mut src, &ctx.config_publisher, &ctx.fast_config_publisher).await;

        if loop_count.is_multiple_of(10) {
            info!("           MSP:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
