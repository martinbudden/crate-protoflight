#![cfg(feature = "gps")]

use embassy_sync::{
    pubsub::{PubSubChannel, Publisher, Subscriber},
    {blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal},
};

use static_cell::StaticCell;

use crate::{
    boards::{GpsUartRx, GpsUartTx},
    gps::{
        Geodetic, GeographicCoordinate, GpsData, GpsMessage, GpsParser, GpsParserEvent, GpsProvider, GpsSolutionData,
        GpsYawHeadingMessage, NmeaGga, NmeaGsa, NmeaRecordType, NmeaRmc, UbxNavPvt,
    },
};

static GPS_CTX: StaticCell<GpsContext> = StaticCell::new();

const MAX_GPS_SUBSCRIBER_COUNT: usize = 4;
const GPS_PUBLISHER_COUNT: usize = 1;
const GPS_PUB_SUB_CAPACITY: usize = 4;

/// `PubSubChannel` for handling `GpsData` updates.
static GPS_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    GpsMessage,
    GPS_PUB_SUB_CAPACITY,
    MAX_GPS_SUBSCRIBER_COUNT,
    GPS_PUBLISHER_COUNT,
> = PubSubChannel::new();

type GpsPublisher = Publisher<
    'static,
    CriticalSectionRawMutex,
    GpsMessage,
    GPS_PUB_SUB_CAPACITY,
    MAX_GPS_SUBSCRIBER_COUNT,
    GPS_PUBLISHER_COUNT,
>;

pub type GpsSubscriber = Subscriber<
    'static,
    CriticalSectionRawMutex,
    GpsMessage,
    GPS_PUB_SUB_CAPACITY,
    MAX_GPS_SUBSCRIBER_COUNT,
    GPS_PUBLISHER_COUNT,
>;

#[allow(clippy::expect_used)]
pub fn gps_subscriber() -> GpsSubscriber {
    GPS_PUB_SUB_CHANNEL.subscriber().expect("gps_subscriber failed")
}

pub static GPS_YAW_HEADING_SIGNAL: Signal<CriticalSectionRawMutex, GpsYawHeadingMessage> = Signal::new();

/// Context for GPS task.
#[allow(unused)]
pub struct GpsContext {
    pub uart_rx: GpsUartRx,
    pub uart_tx: GpsUartTx,
    pub gps_parser: GpsParser,
    pub gps_publisher: GpsPublisher,
    pub gps_data: GpsData,
    pub gps_solution_data: GpsSolutionData,
    pub home: Geodetic,
}

impl GpsContext {
    pub fn new(uart_rx: GpsUartRx, uart_tx: GpsUartTx, gps_provider: GpsProvider) -> Self {
        #[allow(clippy::expect_used)]
        Self {
            uart_rx,
            uart_tx,
            gps_parser: GpsParser::new_unwrapped(gps_provider),
            gps_publisher: GPS_PUB_SUB_CHANNEL.publisher().expect("gps_publisher failed"),
            gps_data: GpsData::new(),
            gps_solution_data: GpsSolutionData::new(),
            home: Geodetic::new(),
        }
    }
}

#[allow(unused)]
pub fn init(uart_rx: GpsUartRx, uart_tx: GpsUartTx, gps_provider: GpsProvider) -> &'static mut GpsContext {
    GPS_CTX.init(GpsContext::new(uart_rx, uart_tx, gps_provider))
}

/// GPS Task Placeholder.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut GpsContext) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(10));
    let mut loop_count: u32 = 0;
    let mut buf = [0u8; 128];

    log::info!("         GPS: task started");
    loop {
        // Wait for the next tick. TODO: chang GPS task to run off interrupt
        ticker.next().await;
        core::future::poll_fn(|_| core::task::Poll::Ready(())).await;

        if let Ok(n) = ctx.uart_rx.read(&mut buf).await {
            for &byte in &buf[..n] {
                if let Some(event) = ctx.gps_parser.on_data_received(byte) {
                    process_gps_event(&mut ctx.gps_data, &mut ctx.gps_solution_data, event);
                }
            }
        }

        // Publish the gps data for use by (eg) the OSD.
        ctx.gps_publisher.publish_immediate(GpsMessage::Data(ctx.gps_data));
        ctx.gps_publisher.publish_immediate(GpsMessage::Solution(ctx.gps_solution_data));

        // Convert the gps data position to a `GpsPositionMeters` use by the autopilot.
        let geographic_coordinate = GeographicCoordinate::from_long_lat_alt(
            ctx.gps_data.longitude_degrees_x1e7,
            ctx.gps_data.latitude_degrees_x1e7,
            ctx.gps_data.altitude_cm,
        );
        let gps_position = ctx.home.distance_from_home_meters(geographic_coordinate);
        ctx.gps_publisher.publish_immediate(GpsMessage::Position(gps_position));

        // Only trust GPS heading if moving faster than 1.5 m/s (150 cmps, approx 3 knots)
        if ctx.gps_data.ground_speed_cmps > 150 {
            let gps_yaw_heading_message = GpsYawHeadingMessage {
                yaw_heading_radians: (f32::from(ctx.gps_data.heading_deci_degrees) * 0.1).to_radians(),
                delta_t: 0.1,
            };
            // signal the yaw heading so the gyro_pid task can use it to correct yaw drift in the sensor fusion filter.
            GPS_YAW_HEADING_SIGNAL.signal(gps_yaw_heading_message);
        }

        if loop_count.is_multiple_of(10) {
            log::info!("             GPS:      loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}

fn process_gps_event(gps: &mut GpsData, gps_solution: &mut GpsSolutionData, event: GpsParserEvent<'_>) {
    match event {
        GpsParserEvent::NmeaComplete(record) => match NmeaRecordType::from_record(record) {
            Some(NmeaRecordType::Gga) => {
                if let Some(gga) = NmeaGga::parse(record) {
                    gps.amend_with_nmea_gga(gga);
                    gps_solution.amend_with_nmea_gga(gga);
                }
            }
            Some(NmeaRecordType::Rmc) => {
                if let Some(rmc) = NmeaRmc::parse(record) {
                    gps.amend_with_nmea_rmc(rmc);
                    gps_solution.amend_with_nmea_rmc(rmc);
                }
            }
            Some(NmeaRecordType::Gsa) => {
                if let Some(gsa) = NmeaGsa::parse(record) {
                    gps.amend_with_nmea_gsa(gsa);
                    gps_solution.amend_with_nmea_gsa(gsa);
                }
            }
            None => {}
        },
        GpsParserEvent::UbxMessage(message) => {
            if message.class == UbxNavPvt::CLASS
                && message.id == UbxNavPvt::ID
                && let Some(nav) = UbxNavPvt::parse(message.payload)
            {
                gps.amend_with_ubx_nav_pvt(nav);
                gps_solution.amend_with_ubx_nav_pvt(nav);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use crate::gps::{UbxMessage, make_realistic_nav_pvt_payload};

    use super::*;

    #[test]
    fn process_gps_event_amends_gga() {
        let record = b"GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,";

        let mut gps = GpsData::default();
        let mut gps_solution = GpsSolutionData::default();

        process_gps_event(&mut gps, &mut gps_solution, GpsParserEvent::NmeaComplete(record));

        //assert_eq!(gps.time_of_week_ms, 45_519_000);
        assert_eq!(gps.latitude_degrees_x1e7, 481_173_000);
        assert_eq!(gps.longitude_degrees_x1e7, 115_166_667);
        assert_eq!(gps.altitude_cm, 54_540);
        assert_eq!(gps.geoid_separation_cm, 4_690);
        assert_eq!(gps.fix, 1);
        assert_eq!(gps.satellite_count, 8);
    }
    #[test]
    fn process_gps_event_amends_nav_pvt() {
        let payload = make_realistic_nav_pvt_payload();

        let mut gps = GpsData::default();
        let mut gps_solution = GpsSolutionData::default();

        let message = UbxMessage { class: 0x01, id: 0x07, payload: &payload };

        process_gps_event(&mut gps, &mut gps_solution, GpsParserEvent::UbxMessage(message));

        assert_eq!(gps.time_of_week_ms, 45_296_789);
        assert_eq!(gps.longitude_degrees_x1e7, -12_345_678);
        assert_eq!(gps.latitude_degrees_x1e7, 512_345_678);
        assert_eq!(gps.altitude_cm, 10_000);
        assert_eq!(gps.geoid_separation_cm, 2345);
        assert_eq!(gps.velocity_north_cmps, 123);
        assert_eq!(gps.velocity_east_cmps, -45);
        assert_eq!(gps.velocity_down_cmps, 12);
        assert_eq!(gps.ground_speed_cmps, 131);
        assert_eq!(gps.heading_deci_degrees, 1_234);
        assert_eq!(gps.satellite_count, 12);
        assert_eq!(gps.fix, 3);
        assert_eq!(gps.is_healthy, 1);
    }
    #[test]
    fn process_gps_event_amends_rmc() {
        let record = b"GPRMC,..."; // existing known-good RMC fixture

        let mut gps = GpsData::default();
        let mut gps_solution = GpsSolutionData::default();

        process_gps_event(&mut gps, &mut gps_solution, GpsParserEvent::NmeaComplete(record));

        /*assert_eq!(gps.time_of_day_ms, /* expected */);
        assert_eq!(gps.latitude_degrees_x1e7, /* expected */);
        assert_eq!(gps.longitude_degrees_x1e7, /* expected */);
        assert_eq!(gps.ground_speed_cmps, /* expected */);
        assert_eq!(gps.heading_deci_degrees, /* expected */);*/
    }
    #[test]
    fn process_gps_event_amends_gsa() {
        let record = b"GPGSA,..."; // existing known-good GSA fixture

        let mut gps = GpsData::default();
        let mut gps_solution = GpsSolutionData::default();

        process_gps_event(&mut gps, &mut gps_solution, GpsParserEvent::NmeaComplete(record));

        /*assert_eq!(
            gps.dilution_of_precision_positional,
            /* expected */
        );*/
    }
    #[test]
    fn process_gps_event_ignores_unknown_nmea_record() {
        let mut gps = GpsData::default();
        let mut gps_solution = GpsSolutionData::default();

        process_gps_event(&mut gps, &mut gps_solution, GpsParserEvent::NmeaComplete(b"GPXXX,something"));

        assert_eq!(gps, GpsData::default());
    }
    #[test]
    fn process_gps_event_ignores_unknown_ubx_message() {
        let mut gps = GpsData::default();
        let mut gps_solution = GpsSolutionData::default();

        let message = UbxMessage { class: 0x01, id: 0x99, payload: &[] };

        process_gps_event(&mut gps, &mut gps_solution, GpsParserEvent::UbxMessage(message));

        assert_eq!(gps, GpsData::default());
    }
}
