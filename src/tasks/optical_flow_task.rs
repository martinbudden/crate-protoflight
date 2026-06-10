#![cfg(feature = "optical_flow")]

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
use log::info;

use crate::sensors::OpticalFlowMessage;

const MAX_OPTICAL_FLOW_SUBSCRIBER_COUNT: usize = 4;
const OPTICAL_FLOW_PUBLISHER_COUNT: usize = 1;
const OPTICAL_FLOW_CAPACITY: usize = 1; // only keep the last item

/// `PubSubChannel` for handling `battery` updates.
static OPTICAL_FLOW_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    OpticalFlowMessage,
    OPTICAL_FLOW_CAPACITY,
    MAX_OPTICAL_FLOW_SUBSCRIBER_COUNT,
    OPTICAL_FLOW_PUBLISHER_COUNT,
> = PubSubChannel::new();

type OpticalFlowPublisher<'a> = Publisher<
    'a,
    CriticalSectionRawMutex,
    OpticalFlowMessage,
    OPTICAL_FLOW_CAPACITY,
    MAX_OPTICAL_FLOW_SUBSCRIBER_COUNT,
    OPTICAL_FLOW_PUBLISHER_COUNT,
>;

pub fn optical_flow_publisher<'a>() -> OpticalFlowPublisher<'a> {
    OPTICAL_FLOW_PUB_SUB_CHANNEL.publisher().expect("optical_flow_publisher failed")
}

#[allow(unused)]
pub type OpticalFlowSubscriber<'a> = Subscriber<
    'a,
    CriticalSectionRawMutex,
    OpticalFlowMessage,
    OPTICAL_FLOW_CAPACITY,
    MAX_OPTICAL_FLOW_SUBSCRIBER_COUNT,
    OPTICAL_FLOW_PUBLISHER_COUNT,
>;

#[allow(unused)]
pub fn optical_flow_subscriber<'a>() -> OpticalFlowSubscriber<'a> {
    OPTICAL_FLOW_PUB_SUB_CHANNEL.subscriber().expect("optical_flow_subscriber failed")
}

/// Context for optical flow task.
pub struct OpticalFlowContext<'a> {
    pub optical_flow_publisher: OpticalFlowPublisher<'a>,
}

/// Optical flow Task Placeholder.
///

#[embassy_executor::task]
pub async fn optical_flow_task(ctx: &'static mut OpticalFlowContext<'static>) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut loop_count: u32 = 0;

    info!("OPTICAL_FLOW: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;

        // TODO: get the battery data by reading the battery.
        let optical_flow_message = OpticalFlowMessage::default();
        ctx.optical_flow_publisher.publish_immediate(optical_flow_message);

        if loop_count.is_multiple_of(10) {
            info!("  OPTICAL_FLOW:  loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
