#![cfg(feature = "optical_flow")]

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};
use static_cell::StaticCell;

use crate::optical_flow_sensors::{OpticalFlow, OpticalFlowMessage, RxOpticalFlow};

const MAX_OPTICAL_FLOW_SUBSCRIBER_COUNT: usize = 4;
const OPTICAL_FLOW_PUBLISHER_COUNT: usize = 1;
const OPTICAL_FLOW_PUB_SUB_CAPACITY: usize = 1; // only keep the last item

/// `PubSubChannel` for handling `battery` updates.
static OPTICAL_FLOW_PUB_SUB_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    OpticalFlowMessage,
    OPTICAL_FLOW_PUB_SUB_CAPACITY,
    MAX_OPTICAL_FLOW_SUBSCRIBER_COUNT,
    OPTICAL_FLOW_PUBLISHER_COUNT,
> = PubSubChannel::new();

type OpticalFlowPublisher = Publisher<
    'static,
    CriticalSectionRawMutex,
    OpticalFlowMessage,
    OPTICAL_FLOW_PUB_SUB_CAPACITY,
    MAX_OPTICAL_FLOW_SUBSCRIBER_COUNT,
    OPTICAL_FLOW_PUBLISHER_COUNT,
>;

pub type OpticalFlowSubscriber = Subscriber<
    'static,
    CriticalSectionRawMutex,
    OpticalFlowMessage,
    OPTICAL_FLOW_PUB_SUB_CAPACITY,
    MAX_OPTICAL_FLOW_SUBSCRIBER_COUNT,
    OPTICAL_FLOW_PUBLISHER_COUNT,
>;

#[allow(clippy::expect_used)]
pub fn optical_flow_subscriber() -> OpticalFlowSubscriber {
    OPTICAL_FLOW_PUB_SUB_CHANNEL.subscriber().expect("optical_flow_subscriber failed")
}

static OPTICAL_FLOW_CTX: StaticCell<OpticalFlowContext> = StaticCell::new();
/// Context for optical flow task.
pub struct OpticalFlowContext {
    pub optical_flow: OpticalFlow,
    pub optical_flow_publisher: OpticalFlowPublisher,
}

impl OpticalFlowContext {
    pub fn new(optical_flow: OpticalFlow) -> Self {
        #[allow(clippy::expect_used)]
        Self {
            optical_flow,
            optical_flow_publisher: OPTICAL_FLOW_PUB_SUB_CHANNEL.publisher().expect("optical_flow_publisher failed"),
        }
    }
}

pub fn init(optical_flow: OpticalFlow) -> &'static mut OpticalFlowContext {
    OPTICAL_FLOW_CTX.init(OpticalFlowContext::new(optical_flow))
}

/// Optical flow Task Placeholder.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut OpticalFlowContext) {
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut loop_count: u32 = 0;

    log::info!("OPTICAL_FLOW: task started");
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;

        let optical_flow_message = ctx.optical_flow.message();
        ctx.optical_flow_publisher.publish_immediate(optical_flow_message);

        if loop_count.is_multiple_of(10) {
            log::info!("  OPTICAL_FLOW:loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
