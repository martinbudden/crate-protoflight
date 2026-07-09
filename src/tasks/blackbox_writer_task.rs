#![cfg(feature = "blackbox")]

#[cfg(feature = "std")]
use crate::drivers::sd_card::{MockSdCard, SdStorage};
use crate::tasks::blackbox_task::BLACKBOX_WRITE_QUEUE;
#[cfg(feature = "rp2350")]
use crate::tasks::init_rp::BlackboxSpiDevice;
#[cfg(not(feature = "std"))]
use embedded_sdmmc::{Directory, Mode, SdCard, VolumeIdx, VolumeManager};

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
#[cfg(not(feature = "std"))]
pub struct BlackboxWriterContext {
    /// Concrete async SPI bus instance assigned to the card subsystem
    pub spi_device: BlackboxSpiDevice,
    /// 512-byte block cache matching the target SD physical sector boundaries
    pub sector_buffer: [u8; Self::BUFFER_SIZE],
    pub buffer_idx: usize,
}

#[cfg(not(feature = "std"))]
impl BlackboxWriterContext {
    const BUFFER_SIZE: usize = 512;
    pub fn new(spi_device: BlackboxSpiDevice) -> Self {
        Self { spi_device, sector_buffer: [0u8; Self::BUFFER_SIZE], buffer_idx: 0 }
    }
}

#[cfg(feature = "std")]
pub struct BlackboxWriterContext {
    pub sd_card: MockSdCard,
}

#[cfg(feature = "std")]
impl BlackboxWriterContext {
    pub fn new() -> Self {
        Self { sd_card: MockSdCard::new("blackbox_log.bbl") }
    }
}

#[cfg(feature = "std")]
#[embassy_executor::task]
pub async fn blackbox_writer_task(ctx: &'static mut BlackboxWriterContext) {
    log::info!("BLACKBOX SD WRITER: task started");
    loop {
        // Asynchronously wait until blackbox_task sends a new serialized block chunk
        let block = BLACKBOX_WRITE_QUEUE.receive().await;
        let chunk = &block.data[..block.len];
        // On desktop, directly await the full file flash operation
        _ = ctx.sd_card.write_all(chunk).await;
    }
}

/// Blackbox writer background processing task loop using embedded-sdmmc 0.9.0.
#[cfg(not(feature = "std"))]
#[embassy_executor::task]
pub async fn blackbox_writer_task(ctx: &'static mut BlackboxWriterContext) {
    log::info!("BLACKBOX SD WRITER: task started");

    // LOW-SPEED BOOT HARDWARE HANDSHAKE ---
    {
        // Mount the card container at the mandatory safe boot speed (400 kHz)
        let sd_card = SdCard::new(&mut ctx.spi_device, embassy_time::Delay);
        let volume_mgr = VolumeManager::new(sd_card, VehicleTimeSource);

        // Open the volume. This underlying library call executes the low-speed
        // handshakes (CMD0, ACMD41) and locks the card hardware into its Transfer State!
        let _volume = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    }

    log::info!("SD CARD: Handshake verified. Shifting master clock registers to 20 MHz...");
    ctx.spi_device.bus_mut().set_frequency(20_000_000);

    // Re-mount the entire framework. Everything from here forward runs at full 20 MHz data rates.
    let sd_card = SdCard::new(&mut ctx.spi_device, embassy_time::Delay);
    let volume_mgr = VolumeManager::new(sd_card, VehicleTimeSource);
    let volume = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    let mut root_dir = volume.open_root_dir().unwrap();

    // Scan directory and generate the log index at 20 MHz speed
    let next_index = find_next_log_index(&mut root_dir);
    let mut filename_buf = [0u8; 12];
    let filename_str = format_log_filename(next_index, &mut filename_buf);

    let log_file = root_dir.open_file_in_dir(filename_str, Mode::ReadWriteCreateOrAppend).unwrap();

    loop {
        // Asynchronously wait until blackbox_task sends a new serialized block chunk
        let block = BLACKBOX_WRITE_QUEUE.receive().await;
        let chunk = &block.data[..block.len];

        // Sector-alignment analysis: Flush working cache to disk if it overflows 512 bytes
        if ctx.buffer_idx + chunk.len() > BlackboxWriterContext::BUFFER_SIZE {
            // Cooperative yield checkpoint: Ensure high-speed control loop runs before physical write
            embassy_time::Timer::after_micros(0).await;

            // Flushes data down at full 20 MHz speed via DMA channels!
            let _ = log_file.write(&ctx.sector_buffer[..ctx.buffer_idx]).unwrap();
            ctx.buffer_idx = 0;
        }

        ctx.sector_buffer[ctx.buffer_idx..ctx.buffer_idx + chunk.len()].copy_from_slice(chunk);
        ctx.buffer_idx += chunk.len();
    }
}

/// Scans the root directory by inspecting raw filename bytes directly.
#[cfg(not(feature = "std"))]
pub fn find_next_log_index<D, T, const DIR: usize, const FILE: usize, const VOL: usize>(
    root_dir: &mut Directory<'_, D, T, DIR, FILE, VOL>,
) -> u16
where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
{
    let mut highest_idx = 0;

    let _ = root_dir.iterate_dir(|entry| {
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
