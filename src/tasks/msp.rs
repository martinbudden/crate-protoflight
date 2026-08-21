#![cfg(feature = "msp")]
#![allow(unused)]

use static_cell::StaticCell;
//use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use stream_buf::{StreamBufReader, StreamBufWriter};

use crate::{
    config::{ConfigPublisher, FastConfigPublisher, config_publisher, fast_config_publisher},
    multiwii_serial_protocol::{Msp, MspSensorData},
};

#[cfg(feature = "barometer")]
use crate::tasks::barometer::{BarometerSubscriber, barometer_subscriber};

#[cfg(feature = "battery")]
use crate::tasks::battery::{BatterySubscriber, battery_subscriber};

#[cfg(feature = "gps")]
use crate::{
    gps::GpsMessage,
    tasks::gps::{GpsSubscriber, gps_subscriber},
};

#[cfg(feature = "magnetometer")]
use crate::tasks::magnetometer::{MagnetometerSubscriber, magnetometer_subscriber};

#[cfg(feature = "optical_flow")]
use crate::tasks::optical_flow::{OpticalFlowSubscriber, optical_flow_subscriber};

#[cfg(feature = "rangefinder")]
use crate::tasks::rangefinder::{RangefinderSubscriber, rangefinder_subscriber};

static MSP_CTX: StaticCell<MspContext> = StaticCell::new();

/// Context for MSP task.
///
pub const MSP_READ_BUF_SIZE: usize = 256;
pub const MSP_WRITE_BUF_SIZE: usize = 512;
pub struct MspContext {
    pub fast_config_publisher: FastConfigPublisher,
    pub config_publisher: ConfigPublisher,
    #[cfg(feature = "barometer")]
    pub barometer_subscriber: BarometerSubscriber,
    #[cfg(feature = "battery")]
    pub battery_subscriber: BatterySubscriber,
    #[cfg(feature = "gps")]
    pub gps_subscriber: GpsSubscriber,
    #[cfg(feature = "magnetometer")]
    pub magnetometer_subscriber: MagnetometerSubscriber,
    #[cfg(feature = "optical_flow")]
    pub optical_flow_subscriber: OpticalFlowSubscriber,
    #[cfg(feature = "rangefinder")]
    pub rangefinder_subscriber: RangefinderSubscriber,
    pub msp: Msp,
    pub read_buf: [u8; MSP_READ_BUF_SIZE],
    pub write_buf: [u8; MSP_WRITE_BUF_SIZE],
}

impl MspContext {
    #[allow(clippy::too_many_arguments)]
    #[rustfmt::skip]
    pub fn new(
    ) -> Self {
        Self {
            msp: Msp::new(),
            fast_config_publisher:fast_config_publisher(),
            config_publisher:config_publisher(),
            #[cfg(feature = "barometer")] barometer_subscriber:barometer_subscriber(),
            #[cfg(feature = "battery")] battery_subscriber:battery_subscriber(),
            #[cfg(feature = "gps")] gps_subscriber:gps_subscriber(),
            #[cfg(feature = "magnetometer")] magnetometer_subscriber:magnetometer_subscriber(),
            #[cfg(feature = "optical_flow")] optical_flow_subscriber:optical_flow_subscriber(),
            #[cfg(feature = "rangefinder")] rangefinder_subscriber:rangefinder_subscriber(),
            read_buf: [0u8; MSP_READ_BUF_SIZE],
            write_buf: [0u8; MSP_WRITE_BUF_SIZE],
        }
    }
}

impl MspContext {
    /// Helper to get a reader for `read_buf`.
    pub fn reader(&'_ mut self) -> StreamBufReader<'_> {
        StreamBufReader::new(&self.read_buf)
    }

    /// Helper to get a writer for `write_buf`.
    pub fn writer(&'_ mut self) -> StreamBufWriter<'_> {
        StreamBufWriter::new(&mut self.write_buf)
    }
}

pub fn init() -> &'static mut MspContext {
    MSP_CTX.init(MspContext::new())
}

/// MSP task Placeholder.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut MspContext) {
    // for now just wait on a ticker to drive the MSP loop. TODO: change this to wait on an MSP packet instead.
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(200));
    let mut loop_count: u32 = 0;

    // value to pass to Msp::process_write_command
    let mut msp_sensor_data = MspSensorData::new();

    log::info!("         MSP: task started");
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
            && let GpsMessage::Data(gps_data) = event
        {
            msp_sensor_data.gps_sol.longitude_degrees_x1e7 = gps_data.longitude_degrees_x1e7;
            msp_sensor_data.gps_sol.satellite_count = gps_data.satellite_count;
            // TODO: check sign of gps data
            #[allow(clippy::cast_sign_loss)]
            {
                msp_sensor_data.gps_sol.ground_speed_cmps = gps_data.ground_speed_cmps as u16;
                msp_sensor_data.gps_sol.ground_course_degrees_x10 = gps_data.heading_deci_degrees as u16;
            }
            msp_sensor_data.gps_sol.pdop = gps_data.pdop_x100;
        }

        // Generally, we don't want to store the Reader itself because it tracks a "cursor" (current position).
        // It's better to store the data and create a fresh reader whenever we start processing a new packet.
        let mut src = StreamBufReader::new(&ctx.read_buf);

        let cmd_msp = Msp::SET_FAILSAFE_CONFIG;
        let _result =
            Msp::process_read_command(cmd_msp, &mut src, &ctx.config_publisher, &ctx.fast_config_publisher).await;

        if loop_count.is_multiple_of(10) {
            log::info!("             MSP:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
