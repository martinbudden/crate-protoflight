#![cfg(feature = "blackbox")]

#[cfg(all(not(feature = "host"), feature = "sdcard"))]
use {
    crate::boards::{SdCardBlockDevice, SdCardTimeSource},
    core::ops::ControlFlow,
    embedded_sdmmc::{Directory, File, Mode},
};

use embassy_futures::yield_now;
use static_cell::StaticCell;

use crate::boards::SdCardVolume;
use crate::tasks::blackbox_encoder::{BLACKBOX_WRITE_QUEUE, BlackboxWriteItem};

/// Type alias for the persistent SD card Directory.
#[cfg(all(not(feature = "host"), feature = "sdcard"))]
pub type SdCardDirectory<'a> = Directory<
    'a,
    SdCardBlockDevice,
    SdCardTimeSource,
    1, // MAX_DIRS
    1, // MAX_FILES
    1, // MAX_VOLUMES
>;
#[cfg(any(feature = "host", not(feature = "sdcard")))]
pub type SdCardDirectory<'a> = ();

// Type alias for the persistent File matching our limits
#[cfg(all(not(feature = "host"), feature = "sdcard"))]
pub type SdCardFile<'a> = File<
    'a,
    SdCardBlockDevice,
    SdCardTimeSource,
    1, // MAX_DIRS
    1, // MAX_FILES
    1, // MAX_VOLUMES
>;
#[cfg(all(not(feature = "host"), not(feature = "sdcard")))]
pub type SdCardFile<'a> = ();
#[cfg(feature = "host")]
pub type SdCardFile<'a> = std::fs::File;

/// System execution context for the blackbox writer.
#[allow(unused)]
pub struct BlackboxWriterContext {
    pub volume: SdCardVolume,
    pub root_dir: SdCardDirectory<'static>,
    pub file: SdCardFile<'static>,
    /// 512-byte cache matching SD physical sector boundaries.
    pub sector_buffer: [u8; Self::SECTOR_SIZE],
    pub sector_idx: usize,
}

impl BlackboxWriterContext {
    const SECTOR_SIZE: usize = 512;
}

static BLACKBOX_WRITER_CTX: StaticCell<BlackboxWriterContext> = StaticCell::new();

/// Attempts to create the blackbox writer context.
/// Returns `None` if the hardware or filesystem fails to mount.
#[cfg(all(not(feature = "host"), feature = "sdcard"))]
pub fn init(volume: SdCardVolume) -> Option<&'static mut BlackboxWriterContext> {
    // Open the root directory directly out of the 'static volume
    let mut root_dir = match volume.open_root_dir() {
        Ok(dir) => dir,
        Err(e) => {
            log::error!("BLACKBOX: Root directory initialization failed: {e:?}");
            return None;
        }
    };

    // Scan directory and generate the log index
    let next_index = find_next_log_index(&mut root_dir);
    let mut filename_buf = [0u8; 12];
    let filename_str = format_log_filename(next_index, &mut filename_buf);

    // Open the file out of the root directory context
    let file = match root_dir.open_file_in_dir(filename_str, Mode::ReadWriteCreateOrAppend) {
        Ok(f) => f,
        Err(e) => {
            log::error!("BLACKBOX: File creation failed: {e:?}");
            return None;
        }
    };

    let ctx = BlackboxWriterContext {
        volume,
        root_dir,
        file,
        sector_buffer: [0u8; BlackboxWriterContext::SECTOR_SIZE],
        sector_idx: 0,
    };

    Some(BLACKBOX_WRITER_CTX.init(ctx))
}

#[cfg(all(not(feature = "host"), not(feature = "sdcard")))]
pub fn init(_volume: SdCardVolume) -> Option<&'static mut BlackboxWriterContext> {
    let ctx = BlackboxWriterContext {
        volume: (),
        root_dir: (),
        file: (),
        sector_buffer: [0u8; BlackboxWriterContext::SECTOR_SIZE],
        sector_idx: 0,
    };

    Some(BLACKBOX_WRITER_CTX.init(ctx))
}

#[cfg(feature = "host")]
pub fn init(_volume: SdCardVolume) -> Option<&'static mut BlackboxWriterContext> {
    use std::fs::OpenOptions;

    log::info!("BLACKBOX: Running on host environment. Target file: blackbox_log.bbl");

    // Open or create the hardcoded file on your local machine
    let file = match OpenOptions::new().read(true).create(true).append(true).open("blackbox_log.bbl") {
        Ok(f) => f,
        Err(e) => {
            log::error!("BLACKBOX HOST ERR: Failed to open/create 'blackbox_log.bbl': {e:?}");
            return None;
        }
    };

    let ctx = BlackboxWriterContext {
        volume: (),
        root_dir: (),
        file,
        sector_buffer: [0u8; BlackboxWriterContext::SECTOR_SIZE],
        sector_idx: 0,
    };

    Some(BLACKBOX_WRITER_CTX.init(ctx))
}

#[embassy_executor::task]
pub async fn run(ctx: &'static mut BlackboxWriterContext) {
    log::info!("BLACKBOX WRITER: task started. Log file is armed.");

    let mut loop_count: u32 = 0;
    loop {
        match BLACKBOX_WRITE_QUEUE.receive().await {
            BlackboxWriteItem::Data(block) => {
                #[cfg(any(feature = "host", feature = "sdcard"))]
                append_to_sector_buffer(
                    &mut ctx.sector_buffer,
                    &mut ctx.sector_idx,
                    &block.data[..block.len],
                    &mut ctx.file,
                );
                yield_now().await;
                if loop_count.is_multiple_of(10) {
                    log::info!(" BLACKBOXw:loop {loop_count},{0}", block.len);
                }
            }
            BlackboxWriteItem::Flush => {
                #[cfg(any(feature = "host", feature = "sdcard"))]
                flush_sector_buffer(&mut ctx.sector_buffer, ctx.sector_idx, &mut ctx.file);
                ctx.sector_idx = 0;
                yield_now().await;
                log::info!(" BLACKBOXf:loop {loop_count}");
                break;
            }
        }
        loop_count = loop_count.wrapping_add(1);
    }
}

#[cfg(any(feature = "host", feature = "sdcard"))]
#[inline]
fn append_to_sector_buffer(
    sector_buffer: &mut [u8; BlackboxWriterContext::SECTOR_SIZE],
    sector_idx: &mut usize,
    chunk: &[u8],
    file: &mut SdCardFile<'static>,
) {
    // Bring std::io::Write into scope ONLY on the host so .write() works exactly like the embedded variant
    #[cfg(feature = "host")]
    use std::io::Write as _;

    let space_remaining = BlackboxWriterContext::SECTOR_SIZE - *sector_idx;

    if chunk.len() <= space_remaining {
        let end = *sector_idx + chunk.len();
        sector_buffer[*sector_idx..end].copy_from_slice(chunk);
        *sector_idx = end;

        if *sector_idx == BlackboxWriterContext::SECTOR_SIZE {
            _ = file.write(sector_buffer);
            *sector_idx = 0;
        }
    } else {
        sector_buffer[*sector_idx..].copy_from_slice(&chunk[..space_remaining]);
        _ = file.write(sector_buffer);

        let remainder = &chunk[space_remaining..];
        sector_buffer[..remainder.len()].copy_from_slice(remainder);
        *sector_idx = remainder.len();
    }
}

#[cfg(any(feature = "host", feature = "sdcard"))]
#[inline]
fn flush_sector_buffer(
    sector_buffer: &mut [u8; BlackboxWriterContext::SECTOR_SIZE],
    sector_idx: usize,
    file: &mut SdCardFile<'static>,
) {
    #[cfg(feature = "host")]
    use std::io::Write as _;
    if sector_idx != 0 {
        sector_buffer[sector_idx..].fill(0);
        _ = file.write(sector_buffer);
    }
    _ = file.flush();
}

/// Scans the root directory by inspecting raw filename bytes.
#[cfg(feature = "sdcard")]
pub fn find_next_log_index<D, T, const DIR: usize, const FILE: usize, const VOL: usize>(
    root_dir: &mut Directory<'_, D, T, DIR, FILE, VOL>,
) -> u16
where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
{
    let mut highest_idx = 0;

    _ = root_dir.iterate_dir(|entry| {
        let base = entry.name.base_name();
        let ext = entry.name.extension();

        // Verify the extension is "BBL" and the base starts with "LOG_"
        if ext == b"BBL" && base.starts_with(b"LOG_") && base.len() >= 7 {
            // Extract the 3 numeric characters from indices 4 to 7.
            if let Ok(num_str) = core::str::from_utf8(&base[4..7])
                && let Ok(idx) = num_str.parse::<u16>()
            {
                highest_idx = highest_idx.max(idx);
            }
        }
        ControlFlow::Continue(())
    });
    if highest_idx >= 999 { 0 } else { highest_idx + 1 }
}

/// Helper function to perform pure ASCII modifications safely inside stack boundaries.
#[allow(unused)]
fn format_log_filename(index: u16, buf: &mut [u8; 12]) -> &str {
    buf[0..4].copy_from_slice(b"LOG_");
    buf[7..12].copy_from_slice(b".BBL");
    buf[4] = ((index / 100) % 10) as u8 + b'0';
    buf[5] = ((index / 10) % 10) as u8 + b'0';
    buf[6] = (index % 10) as u8 + b'0';
    core::str::from_utf8(buf).unwrap_or("LOG_000.BBL")
}
