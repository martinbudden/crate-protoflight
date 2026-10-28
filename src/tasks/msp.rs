#![cfg(feature = "msp")]
#![allow(unused)]

use {
    static_cell::StaticCell,
    stream_buf::{StreamBufReader, StreamBufWriter},
};

use crate::{
    boards::{UsbReceiver, UsbSender},
    config::{ConfigPublisher, FastConfigPublisher, config_publisher, fast_config_publisher},
    multiwii_serial_protocol::{Msp, MspParser, MspPort, MspSensorData, MspStream},
    tasks::errors::TaskInitError,
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
    pub msp_sensor_data: MspSensorData,

    pub msp_parser: MspParser,
    pub msp_port: MspPort,
}

pub fn init(usb_sender: UsbSender, usb_receiver: UsbReceiver) -> Result<&'static mut MspContext, TaskInitError> {
    #[rustfmt::skip]
    let ctx = MspContext {
        msp_parser: MspParser::new(),
        fast_config_publisher: fast_config_publisher()?,
        config_publisher: config_publisher()?,

        #[cfg(feature = "barometer")] barometer_subscriber: barometer_subscriber()?,
        #[cfg(feature = "battery")] battery_subscriber: battery_subscriber()?,
        #[cfg(feature = "gps")] gps_subscriber: gps_subscriber()?,
        #[cfg(feature = "magnetometer")] magnetometer_subscriber: magnetometer_subscriber()?,
        #[cfg(feature = "optical_flow")] optical_flow_subscriber: optical_flow_subscriber()?,
        #[cfg(feature = "rangefinder")] rangefinder_subscriber: rangefinder_subscriber()?,

        msp_sensor_data:MspSensorData::new(),
        msp_port: MspPort::new(usb_sender, usb_receiver),
    };
    Ok(MSP_CTX.init(ctx))
}

/// MSP task Placeholder.
#[allow(clippy::unused_async)]
#[embassy_executor::task]
pub async fn run(ctx: &'static mut MspContext) {
    // for now just wait on a ticker to drive the MSP loop. TODO: change this to wait on an MSP packet instead.
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(200));
    let mut loop_count: u32 = 0;

    log::info!("         MSP: task started");
    /*#[cfg(feature = "usb")]
    loop {
        ctx.usb_receiver.wait_connection().await;

        while let Ok(n) = ctx.usb_receiver.read_packet(&mut ctx.read_buf).await {
            // Feed bytes into the MSP parser state-machine
            let mut option = None;
            for &byte in &ctx.read_buf[..n] {
                option = ctx.msp_parser.on_data_received(byte);
                if option.is_some() {
                    break;
                }
            }
            if let Some(msp_packet) = option {
                // Process command and serialize response into tx_buf
                //let response_len = process_msp_command(&msp_packet, &mut ctx.write_buf);

                // Generally, we don't want to store the StreamBufReader itself because it tracks a "cursor" (current position).
                // It's better to store the data and create a fresh StreamBufReader whenever we start processing a new packet.
                let mut src = StreamBufReader::new(&ctx.msp_parser.buf);

                let _result = Msp::process_read_command(
                    msp_packet.cmd,
                    &mut src,
                    &ctx.config_publisher,
                    &ctx.fast_config_publisher,
                )
                .await;

                /*// Send response immediately
                if response_len > 0 {
                    if ctx.usb_sender.write_packet(&ctx.write_buf[..response_len]).await.is_err() {
                        // Connection dropped, fall back to wait_connection
                        break;
                    }
                }*/
            }
        }
    }*/
    loop {
        // Wait for msp packet
        // let msp_packet = msp.receive().await;
        ticker.next().await; // for now just wait on ticker

        ctx.msp_port.serial_process(&ctx.fast_config_publisher, &ctx.config_publisher, &ctx.msp_sensor_data).await;

        #[cfg(feature = "barometer")]
        if let Some(wait_result) = ctx.barometer_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(barometer_data) = wait_result
        {
            #[allow(clippy::cast_possible_truncation)]
            {
                ctx.msp_sensor_data.barometer_altitude_cm =
                    ((barometer_data.altitude_m * 100.0) as i32).cast_unsigned();
            }
        }

        #[cfg(feature = "rangefinder")]
        if let Some(wait_result) = ctx.rangefinder_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(rangefinder_message) = wait_result
        {
            #[allow(clippy::cast_possible_truncation)]
            {
                ctx.msp_sensor_data.rangefinder_altitude_cm =
                    ((rangefinder_message.distance_m * 100.0) as i32).cast_unsigned();
            }
        }

        #[cfg(feature = "gps")]
        if let Some(wait_result) = ctx.gps_subscriber.try_next_message()
            && let embassy_sync::pubsub::WaitResult::Message(event) = wait_result
            && let GpsMessage::Data(gps_data) = event
        {
            ctx.msp_sensor_data.gps_solution.longitude_degrees_x1e7 = gps_data.longitude_degrees_x1e7;
            ctx.msp_sensor_data.gps_solution.satellite_count = gps_data.satellite_count;
            // TODO: check sign of gps data
            #[allow(clippy::cast_sign_loss)]
            {
                ctx.msp_sensor_data.gps_solution.ground_speed_cmps = gps_data.ground_speed_cmps as u16;
                ctx.msp_sensor_data.gps_solution.ground_course_degrees_x10 = gps_data.heading_deci_degrees as u16;
            }
            ctx.msp_sensor_data.gps_solution.pdop = gps_data.pdop_x100;
        }

        if loop_count.is_multiple_of(10) {
            log::info!("             MSP:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1);
    }
}
