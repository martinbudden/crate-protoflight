#![cfg(feature = "blackbox")]

use static_cell::StaticCell;

use crate::tasks::blackbox_encoder::{BLACKBOX_WRITE_QUEUE, BlackboxWriteItem};

#[cfg(feature = "rp2350")]
use {
    //crate::boards::rp2350::BlackboxSpiDevice,
    embedded_sdmmc::{Directory, Mode, SdCard, VolumeIdx, VolumeManager},
};

#[cfg(feature = "std")]
use crate::drivers::sd_card::{MockSdCard, SdStorage};

/// Dummy time source required by the embedded-sdmmc library
#[cfg(not(feature = "std"))]
pub struct VehicleTimeSource;

#[cfg(not(feature = "std"))]
impl embedded_sdmmc::TimeSource for VehicleTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        // Returns a fixed default time; can be mapped to an RTC later
        embedded_sdmmc::Timestamp::from_fat(0, 0)
    }
}
/// System execution context for the background storage worker pipeline.
pub struct BlackboxWriterContext {
    #[cfg(feature = "std")]
    pub sd_card: MockSdCard,
    #[cfg(feature = "rp2350")]
    pub spi_device: BlackboxSpiDevice,
    /// 512-byte cache matching SD physical sector boundaries.
    pub sector_buffer: [u8; Self::SECTOR_SIZE],
    pub sector_idx: usize,
}

// A single BlackboxWriteBlock must always fit within one SD sector.
const _: () =
    assert!(crate::tasks::blackbox_encoder::BlackboxWriteBlock::CAPACITY <= BlackboxWriterContext::SECTOR_SIZE);

impl BlackboxWriterContext {
    const SECTOR_SIZE: usize = 512;

    #[cfg(feature = "std")]
    pub fn new() -> Self {
        Self { sd_card: MockSdCard::new("blackbox_log.bbl"), sector_buffer: [0u8; Self::SECTOR_SIZE], sector_idx: 0 }
    }
    #[cfg(feature = "rp2350")]
    pub fn new() -> Self {
        Self { spi_device, sector_buffer: [0u8; Self::SECTOR_SIZE], sector_idx: 0 }
    }
    #[cfg(feature = "stm32")]
    pub fn new() -> Self {
        Self { sector_buffer: [0u8; Self::SECTOR_SIZE], sector_idx: 0 }
    }
}

static BLACKBOX_WRITER_CTX: StaticCell<BlackboxWriterContext> = StaticCell::new();

pub fn init() -> &'static mut BlackboxWriterContext {
    BLACKBOX_WRITER_CTX.init(BlackboxWriterContext::new())
}

#[embassy_executor::task]
pub async fn run(ctx: &'static mut BlackboxWriterContext) {
    log::info!("BLACKBOX WRITER: task started");

    open_storage();

    loop {
        match BLACKBOX_WRITE_QUEUE.receive().await {
            BlackboxWriteItem::Data(block) => {
                let chunk = &block.data[..block.len];
                append_to_sector_buffer(ctx, chunk).await;
            }
            BlackboxWriteItem::Flush => {
                flush_sector_buffer(ctx).await;
                break;
            }
        }
    }
}

async fn append_to_sector_buffer(ctx: &mut BlackboxWriterContext, chunk: &[u8]) {
    let space_remaining = BlackboxWriterContext::SECTOR_SIZE - ctx.sector_idx;

    if chunk.len() <= space_remaining {
        // Entire chunk fits in the current sector.
        let end = ctx.sector_idx + chunk.len();
        ctx.sector_buffer[ctx.sector_idx..end].copy_from_slice(chunk);
        ctx.sector_idx = end;
        // If exactly full, write the sector.
        if ctx.sector_idx == BlackboxWriterContext::SECTOR_SIZE {
            let _ = ctx.sd_card.write_all(&ctx.sector_buffer).await;
            ctx.sector_idx = 0;
        }
    } else {
        // Chunk crosses the sector boundary.
        // Fill the remainder of the current sector.
        ctx.sector_buffer[ctx.sector_idx..].copy_from_slice(&chunk[..space_remaining]);
        let _ = ctx.sd_card.write_all(&ctx.sector_buffer).await;
        // Copy the remainder of the chunk into the new sector.
        let remainder = &chunk[space_remaining..];
        ctx.sector_buffer[..remainder.len()].copy_from_slice(remainder);
        ctx.sector_idx = remainder.len();
    }
}

async fn flush_sector_buffer(ctx: &mut BlackboxWriterContext) {
    if ctx.sector_idx != 0 {
        // Pad the rest of the sector with zeros.
        ctx.sector_buffer[ctx.sector_idx..].fill(0);
        _ = ctx.sd_card.write_all(&ctx.sector_buffer).await;
        ctx.sector_idx = 0;
    }

    ctx.sd_card.flush().await;
}

#[cfg(not(feature = "rp2350"))]
fn open_storage() {}

#[cfg(feature = "rp2350")]
fn open_storage() {
    // TODO: add spi_device parameter to open_storage
    // LOW-SPEED BOOT HARDWARE HANDSHAKE ---
    {
        // Mount the card container at the mandatory safe boot speed (400 kHz)
        let sd_card = SdCard::new(&mut spi_device, embassy_time::Delay);
        let volume_mgr = VolumeManager::new(sd_card, VehicleTimeSource);

        // Open the volume. This underlying library call executes the low-speed
        // handshakes (CMD0, ACMD41) and locks the card hardware into its Transfer State!
        let _volume = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    }

    log::info!("SD CARD: Handshake verified. Shifting master clock registers to 20 MHz...");
    spi_device.bus_mut().set_frequency(20_000_000);

    // Re-mount the entire framework. Everything from here forward runs at full 20 MHz data rates.
    let sd_card = SdCard::new(&mut spi_device, embassy_time::Delay);
    let volume_mgr = VolumeManager::new(sd_card, VehicleTimeSource);
    let volume = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    let mut root_dir = volume.open_root_dir().unwrap();

    // Scan directory and generate the log index at 20 MHz speed
    let next_index = find_next_log_index(&mut root_dir);
    let mut filename_buf = [0u8; 12];
    let filename_str = format_log_filename(next_index, &mut filename_buf);

    let log_file = root_dir.open_file_in_dir(filename_str, Mode::ReadWriteCreateOrAppend).unwrap();
}

/// Scans the root directory by inspecting raw filename bytes directly.
#[cfg(feature = "rp2350")]
pub fn find_next_log_index<D, T, const DIR: usize, const FILE: usize, const VOL: usize>(
    root_dir: &mut Directory<'_, D, T, DIR, FILE, VOL>,
) -> u16
where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
{
    let mut highest_idx = 0;

    _ = root_dir.iterate_dir(|entry| {
        let base = entry.name.base_name(); // Returns &[u8; 8]
        let ext = entry.name.extension(); // Returns &[u8; 3]

        // 1. Verify the extension matches "BIN"
        if ext == b"BIN" {
            // 2. Verify the base starts with "LOG_"
            if &base[0..4] == b"LOG_" {
                // 3. Extract the 3 numeric characters from indices 4 to 7 safely
                if let Ok(num_str) = core::str::from_utf8(&base[4..7]) {
                    if let Ok(idx) = u16::from_str_radix(num_str, 10) {
                        if idx > highest_idx {
                            highest_idx = idx;
                        }
                    }
                }
            }
        }
    });

    if highest_idx >= 999 { 0 } else { highest_idx + 1 }
}

/// Helper function to perform pure ASCII modifications safely inside stack boundaries
#[cfg(not(feature = "std"))]
fn format_log_filename(index: u16, buf: &mut [u8; 12]) -> &str {
    buf[0..4].copy_from_slice(b"LOG_");
    buf[7..12].copy_from_slice(b".BIN");
    buf[4] = ((index / 100) % 10) as u8 + b'0';
    buf[5] = ((index / 10) % 10) as u8 + b'0';
    buf[6] = (index % 10) as u8 + b'0';
    core::str::from_utf8(buf).unwrap_or("LOG_000.BIN")
}
