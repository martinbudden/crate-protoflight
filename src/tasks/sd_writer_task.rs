#![cfg(all(feature = "blackbox", feature = "rp2350"))]

use crate::tasks::{init_rp::BlackboxSpiDevice,
blackbox_task::BLACKBOX_WRITE_QUEUE};
use embedded_sdmmc::{SdCard, VolumeManager, Mode, VolumeIdx, Directory};

/// Dummy time source required by the embedded-sdmmc library
pub struct VehicleTimeSource;
impl embedded_sdmmc::TimeSource for VehicleTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        // Returns a fixed default time; can be mapped to an RTC later
        embedded_sdmmc::Timestamp::from_fat(0, 0)
    }
}

/// System execution context for the background storage worker pipeline.
pub struct SdWriterContext {
    /// Concrete async SPI bus instance assigned to the card subsystem
    pub spi_device: BlackboxSpiDevice,
    /// 512-byte block cache matching the target SD physical sector boundaries
    pub sector_buffer: [u8; 512],
    /// Current pointer offset tracker within the sector cache array
    pub buffer_idx: usize,
}

impl SdWriterContext {
    pub fn new(spi_device: BlackboxSpiDevice) -> Self {
        Self {
            spi_device,
            sector_buffer: [0u8; 512],
            buffer_idx: 0,
        }
    }
}

/// SD Writer background processing task loop using embedded-sdmmc 0.9.0.
#[embassy_executor::task]
pub async fn sd_writer_task(ctx: &'static mut SdWriterContext) {
    log::info!("BLACKBOX SD WRITER: task started");

    // 1. Mount the block driver container
    let sd_card = SdCard::new(&mut ctx.spi_device, embassy_time::Delay);
    let volume_mgr = VolumeManager::new(sd_card, VehicleTimeSource);
    
    // 2. Open partition volume directly from the manager
    let volume = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    
    // 3. Open the root directory from the active volume context
    let mut root_dir = volume.open_root_dir().unwrap();

    // 4. Scan directory using the 0.9.0 object syntax layout
    let next_index = find_next_log_index(&mut root_dir);

    // 5. Format filename into standard 8.3 FAT layout entirely within stack memory boundaries
    let mut filename_buf = [0u8; 12];
    let filename_str = format_log_filename(next_index, &mut filename_buf);
    
    // 6. Open or create the file directly on the root directory object
    let log_file = root_dir
        .open_file_in_dir(filename_str, Mode::ReadWriteCreateOrAppend)
        .unwrap();

    loop {
        // Asynchronously wait until blackbox_task sends a new serialized block chunk.
        let block = BLACKBOX_WRITE_QUEUE.receive().await;
        let chunk = &block.data[..block.len];

        // 7. Sector-alignment analysis: If incoming data chunk overflows 512 bytes, 
        // flush the current working buffer block directly onto physical flash storage first.
        if ctx.buffer_idx + chunk.len() > 512 {
            
            // Cooperative yield checkpoint: Ensure high-speed control loop runs before SPI lockup
            embassy_time::Timer::after_micros(0).await;

            // FIX: write() is now called directly on the File handle object instance
            let _ = log_file.write(&ctx.sector_buffer[..ctx.buffer_idx]).unwrap();
            ctx.buffer_idx = 0;
        }

        // 8. Extract block chunk contents and copy payload into context workspace memory
        ctx.sector_buffer[ctx.buffer_idx..ctx.buffer_idx + chunk.len()].copy_from_slice(chunk);
        ctx.buffer_idx += chunk.len();
    }
}

/// Scans the root directory by inspecting raw filename bytes directly.
pub fn find_next_log_index<D, T, const DIR: usize, const FILE: usize, const VOL: usize>(
    root_dir: &mut Directory<'_, D, T, DIR, FILE, VOL>
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
fn format_log_filename(index: u16, buf: &mut [u8; 12]) -> &str {
    buf[0..4].copy_from_slice(b"LOG_");
    buf[7..12].copy_from_slice(b".BIN");
    buf[4] = ((index / 100) % 10) as u8 + b'0';
    buf[5] = ((index / 10) % 10) as u8 + b'0';
    buf[6] = (index % 10) as u8 + b'0';
    core::str::from_utf8(buf).unwrap_or("LOG_000.BIN")
}
