#![cfg(feature = "gps")]

use log::info;

use crate::{
    gps::{Geodetic, GeographicCoordinate, GpsSolutionData},
    gps::{
        GpsDataItem, {GpsData, GpsPosition, GpsYawHeadingData},
    },
};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
const MAX_GPS_DATA_SUBSCRIBER_COUNT: usize = 8;
const GPS_DATA_PUBLISHER_COUNT: usize = 1;
const GPS_DATA_CAPACITY: usize = 4;

/// `PubSubChannel` for handling `GpsData` updates.
static GPS_DATA_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    GpsDataItem,
    GPS_DATA_CAPACITY,
    MAX_GPS_DATA_SUBSCRIBER_COUNT,
    GPS_DATA_PUBLISHER_COUNT,
> = PubSubChannel::new();

pub type GpsDataPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    GpsDataItem,
    GPS_DATA_CAPACITY,
    MAX_GPS_DATA_SUBSCRIBER_COUNT,
    GPS_DATA_PUBLISHER_COUNT,
>;

pub fn gps_data_publisher<'a>() -> GpsDataPublisher<'a> {
    GPS_DATA_PUB_SUB_CHANNEL.publisher().expect("sensor_data_publisher failed")
}

pub type GpsDataSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    GpsDataItem,
    GPS_DATA_CAPACITY,
    MAX_GPS_DATA_SUBSCRIBER_COUNT,
    GPS_DATA_PUBLISHER_COUNT,
>;

pub fn gps_data_subscriber<'a>() -> GpsDataSubscriber<'a> {
    GPS_DATA_PUB_SUB_CHANNEL.subscriber().expect("sensor_data_subscriber failed")
}

/// The only subscriber is the `gyro_pid_task`.
const MAX_YAW_HEADING_SUBSCRIBER_COUNT: usize = 1;
const YAW_HEADING_PUBLISHER_COUNT: usize = 1;
const YAW_HEADING_CAPACITY: usize = 1;

/// High speed `PubSubChannel` for handling `GpsData` updates in the  `gyro_pid` task.
static YAW_HEADING_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    GpsYawHeadingData,
    YAW_HEADING_CAPACITY,
    MAX_YAW_HEADING_SUBSCRIBER_COUNT,
    YAW_HEADING_PUBLISHER_COUNT,
> = PubSubChannel::new();

pub type YawHeadingPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    GpsYawHeadingData,
    YAW_HEADING_CAPACITY,
    MAX_YAW_HEADING_SUBSCRIBER_COUNT,
    YAW_HEADING_PUBLISHER_COUNT,
>;

pub fn yaw_heading_publisher<'a>() -> YawHeadingPublisher<'a> {
    YAW_HEADING_PUB_SUB_CHANNEL.publisher().expect("yaw_heading_publisher failed")
}

pub type YawHeadingSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    GpsYawHeadingData,
    YAW_HEADING_CAPACITY,
    MAX_YAW_HEADING_SUBSCRIBER_COUNT,
    YAW_HEADING_PUBLISHER_COUNT,
>;

pub fn yaw_heading_subscriber<'a>() -> YawHeadingSubscriber<'a> {
    YAW_HEADING_PUB_SUB_CHANNEL.subscriber().expect("yaw_heading_subscriber failed")
}

/// Context for GPS task.
pub struct GpsContext<'a> {
    pub yaw_heading_publisher: YawHeadingPublisher<'a>,
    pub gps_data_publisher: GpsDataPublisher<'a>,
    pub home: Geodetic,
}

/// GPS Task Placeholder.
#[embassy_executor::task]
pub async fn gps_task(ctx: &'static mut GpsContext<'static>) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(10));
    let mut loop_count: u32 = 0;

    info!("GPS: task started");
    loop {
        // Wait for the next tick.
        ticker.next().await;

        // TODO: this should get the data from the actual GPS sensor.
        let gps_data = GpsData::default();
        let gps_solution = GpsSolutionData::default();

        // Publish the raw gps data for use by (eg) the OSD.
        ctx.gps_data_publisher.publish_immediate(GpsDataItem::Gps(gps_data));
        ctx.gps_data_publisher.publish_immediate(GpsDataItem::GpsSolution(gps_solution));

        // Convert the gps_data position to a GpsPosition item (ie position in meters from home) for use by the autopilot.
        let geographic_coordinate = GeographicCoordinate::from(gps_data.position);
        let gps_position = GpsPosition { position: ctx.home.distance_from_home_meters(geographic_coordinate) };
        ctx.gps_data_publisher.publish_immediate(GpsDataItem::GpsPosition(gps_position));

        // Only trust GPS heading if moving faster than 1.5 m/s (150 cmps, approx 3 knots)
        if gps_data.ground_speed_cmps > 150 {
            let gps_yaw_heading_data = GpsYawHeadingData {
                yaw_heading_radians: (f32::from(gps_data.heading_deci_degrees) * 0.1).to_radians(),
                delta_t: 0.1,
            };
            // publish the yaw heading so the gyro_pid task can use it to correct yaw drift in the sensor fusion filter.
            ctx.yaw_heading_publisher.publish_immediate(gps_yaw_heading_data);
        }

        if loop_count.is_multiple_of(10) {
            info!("      GPS:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
