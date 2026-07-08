#![cfg(all(feature = "blackbox", feature = "rp2350"))]
#![allow(unused)]

use async_embedded_sdmmc::{Mode, SdCard, VolumeIdx, VolumeManager};
pub struct SdWriterContext<'a> {
    pub buffer: [u8; 1024],
    pub pos: usize,
    //pub slice_writer: SliceEncoder<'static>,
    pub _phantom: core::marker::PhantomData<&'a ()>,
}

/// SD Writer task placeholder.
#[embassy_executor::task]
pub async fn sd_writer_task(ctx: &'static mut SdWriterContext<'static>) {
    log::info!("SD_WRITER: task started");
    // TODO: remove ticker once we have a proper async SD card driver that can be awaited on.
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_hz(50));
    let mut time_us: u32 = 0;
    let mut loop_count: u32 = 0;

    loop_count = 0;
    loop {
        // Wait for the next 50Hz tick.
        ticker.next().await;
        time_us = time_us.wrapping_add(125);
        if loop_count.is_multiple_of(10) {
            log::info!("     SD_WRITER: loop {loop_count}");
        }
        loop_count = loop_count.wrapping_add(1); // use wrapping_add to handle when time rolls over at max u32.
    }
}
