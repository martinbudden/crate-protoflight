#![cfg(all(feature = "blackbox", feature = "rp2350"))]
#![allow(unused)]

//use async_embedded_sdmmc::{Mode, SdCard, VolumeIdx, VolumeManager, Directory, BlockDevice, TimeSource};

pub struct SdWriterContext<'a> {
    pub buffer: [u8; 1024],
    pub pos: usize,
    //pub slice_writer: SliceEncoder<'static>,
    pub _phantom: core::marker::PhantomData<&'a ()>,
}

/*/// Scans the root directory asynchronously using async-embedded-sdmmc traits.
/// Detects the highest active index matching "LOG_*.BIN" and outputs the next number.
pub async fn find_next_log_index<D, T>(
    volume_mgr: &mut VolumeManager<D, T>,
    root_dir: &Directory
) -> u16
where
    D: BlockDevice,
    T: TimeSource
{
    let mut highest_idx = 0;

    // FIX: iterate_dir is now an async operation.
    // The closure processes each directory entry matching standard 8.3 FAT criteria.
    let _ = volume_mgr.iterate_dir(root_dir, |entry| {
        // Safe stack-allocated string presentation layer from the library
        let name = entry.name.to_string();

        // Target names exactly following our naming blueprint: "LOG_NNN.BIN"
        if name.starts_with("LOG_") && name.ends_with(".BIN") {
            // Extract the dynamic slice offset bounds
            if let Some(num_str) = name.get(4..7) {
                if let Ok(idx) = parse_u16_from_str(num_str) {
                    if idx > highest_idx {
                        highest_idx = idx;
                    }
                }
            }
        }
    }).await; // Crucial async yield checkpoint

    // Increment index safely, wrapping if it hits the maximum 3-digit constraint (999)
    if highest_idx >= 999 {
        0
    } else {
        highest_idx + 1
    }
}*/

/// Lightweight numeric string parser operating entirely without core allocation assets
fn parse_u16_from_str(s: &str) -> Result<u16, ()> {
    let mut res = 0u16;
    for c in s.chars() {
        if c.is_ascii_digit() {
            res = res * 10 + (c as u16 - '0' as u16);
        } else {
            return Err(());
        }
    }
    Ok(res)
}
/// Formats a number into a fixed layout buffer: "LOG_000.BIN"
fn format_log_filename(index: u16, buf: &mut [u8; 12]) -> &str {
    // Fill buffer with template framework
    buf[0..4].copy_from_slice(b"LOG_");
    buf[7..12].copy_from_slice(b".BIN");

    // Extract digits manually into ASCII representations
    let hundreds = ((index / 100) % 10) as u8 + b'0';
    let tens = ((index / 10) % 10) as u8 + b'0';
    let ones = (index % 10) as u8 + b'0';

    buf[4] = hundreds;
    buf[5] = tens;
    buf[6] = ones;

    // Safe conversion back to basic string reference
    core::str::from_utf8(buf).unwrap_or("LOG_000.BIN")
}

// Dummy timestamp provider required by the library interface
/*struct DummyTimeSource;
impl async_embedded_sdmmc::TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> async_embedded_sdmmc::Timestamp {
        async_embedded_sdmmc::Timestamp::from_fat(0, 0)
    }
}*/
/// SD Writer task placeholder.
#[allow(unused)]
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

/*#[embassy_executor::task]
pub async fn sd_writer_task(mut spi_device: AnySpiDevice<'static>) {
    // The type-erased AnySpiDevice matches the traits expected by async-embedded-sdmmc
    let sd_card = SdCard::new(spi_device, embassy_time::Delay);
    let mut volume_mgr = VolumeManager::new(sd_card, DummyTimeSource);

    let mut volume = volume_mgr.open_volume(VolumeIdx(0)).await.unwrap();
    let mut root_dir = volume.open_root_dir().await.unwrap();

    let mut log_file = root_dir
        .open_file_in_dir("blackbox.bin", Mode::ReadWriteCreateOrAppend)
        .await
        .unwrap();

    let mut sector_buffer = [0u8; 512];
    let mut buffer_idx = 0;

    /*loop {
        // Asynchronously pull telemetry frames from the high-speed channel
        let frame = LOG_QUEUE.receive().await;
        let mut serialize_buf = [0u8; 64];

        if let Ok(serialized) = postcard::to_slice(&frame, &mut serialize_buf) {
            // Sector-alignment check to guarantee reliable 1kHz performance
            if buffer_idx + serialized.len() > 512 {
                let _ = log_file.write(&sector_buffer[..buffer_idx]).await;
                buffer_idx = 0;
            }
            sector_buffer[buffer_idx..buffer_idx + serialized.len()].copy_from_slice(serialized);
            buffer_idx += serialized.len();
        }
    }*/
}
*/
