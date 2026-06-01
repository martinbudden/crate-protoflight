#![allow(unused)]

use log::info;

use crate::{
    gps::{self, Geodetic, GeographicCoordinate},
    sensor_data::{FastSensorDataItem, FastSensorDataPublisher, SensorDataItem, SensorDataPublisher},
    sensors::{GpsData, GpsPosition, GpsYawHeadingData},
};

pub(crate) static GPS_CTX: static_cell::StaticCell<GpsContext> = static_cell::StaticCell::new();

/// Context for GPS task.
pub struct GpsContext<'a> {
    pub fast_sensor_data_publisher: FastSensorDataPublisher<'a>,
    pub sensor_data_publisher: SensorDataPublisher<'a>,
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

        // Publish the raw gps data for use by (eg) the OSD.
        ctx.sensor_data_publisher.publish_immediate(SensorDataItem::Gps(gps_data));

        // Convert the gps_data position to a GpsPosition item (ie position in meters from home) for use by the autopilot.
        let geographic_coordinate = GeographicCoordinate::from(gps_data.position);
        let gps_position = GpsPosition { position: ctx.home.distance_from_home_meters(geographic_coordinate) };
        ctx.sensor_data_publisher.publish_immediate(SensorDataItem::GpsPosition(gps_position));

        // Only trust GPS heading if moving faster than 1.5 m/s (150 cmps, approx 3 knots)
        if gps_data.ground_speed_cmps > 150 {
            let gps_yaw_heading_data = GpsYawHeadingData {
                yaw_heading_radians: (f32::from(gps_data.heading_deci_degrees) * 0.1).to_radians(),
                delta_t: 0.1,
            };
            // publish the yaw heading so the gyro_pid task can use it to correct yaw drift in the sensor fusion filter.
            ctx.fast_sensor_data_publisher.publish_immediate(FastSensorDataItem::GpsYawHeading(gps_yaw_heading_data));
        }

        if loop_count.is_multiple_of(10) {
            info!("      GPS:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
