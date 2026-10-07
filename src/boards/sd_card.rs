#![cfg(feature = "sdcard")]

use embassy_time::Delay;
#[allow(unused)]
use embedded_hal::delay::DelayNs;
use embedded_sdmmc::{SdCard, TimeSource, Timestamp, Volume, VolumeIdx, VolumeManager};
use static_cell::StaticCell;

use super::targets::SdCardSpiDevice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageVolumeError {
    LowSpeedHandshakeFailed,
    HighSpeedReopenFailed,
}
/*
    Err(StorageError::LowSpeedHandshakeFailed) => {
        log::error!("Blackbox Critical: No SD card detected. Lighting up warning LED.");
        // Enter a degraded mode, or try to alert the operator via a secondary bus
    }
    Err(StorageError::HighSpeedReopenFailed) => {
        log::warn!("Signal integrity bad at 20 MHz. Falling back and retrying at 10 MHz...");
        // You could pass a target frequency parameter to open_storage to retry slower!
    }
*/

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    NotImplemented,
    FileSystemCorruptedOrMissing,
    LogFileCreationFailed,
}

/*/// Dummy time source required by the embedded-sdmmc library.
#[allow(unused)]
#[cfg(not(feature = "std"))]
pub struct VehicleTimeSource;

#[cfg(not(feature = "std"))]
impl embedded_sdmmc::TimeSource for VehicleTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        // Returns a fixed default time.
        // TODO: map Blackbox timestamp to a RTC
        embedded_sdmmc::Timestamp::from_fat(0, 0)
    }
}
*/

/// A dummy time source, which is mostly important for creating files.
#[derive(Default)]
pub struct SdCardTimeSource;

impl TimeSource for SdCardTimeSource {
    // In theory you could use the RTC of the rp pico here, if you had any external time synchronizing device.
    fn get_timestamp(&self) -> Timestamp {
        Timestamp { year_since_1970: 0, zero_indexed_month: 0, zero_indexed_day: 0, hours: 0, minutes: 0, seconds: 0 }
    }
}

pub type SdCardBlockDevice = SdCard<&'static mut SdCardSpiDevice, Delay>;

pub type SdCardVolume = Volume<
    'static,
    SdCardBlockDevice,
    SdCardTimeSource,
    1, // MAX_DIRS
    1, // MAX_FILES
    1, // MAX_VOLUMES
>;

// Type alias for the VolumeManager to match the const generics exactly
type ConcreteVolumeManager = VolumeManager<SdCardBlockDevice, SdCardTimeSource, 1, 1, 1>;

static VOLUME_MGR_CELL: StaticCell<ConcreteVolumeManager> = StaticCell::new();

pub fn open_volume(spi_device: &mut SdCardSpiDevice) -> Result<SdCardVolume, StorageVolumeError> {
    // LOW-SPEED BOOT HARDWARE HANDSHAKE ---
    {
        // FIX: Create a fresh short-lived re-borrow instead of moving the pointer entirely
        let sd_card = SdCard::new(&mut *spi_device, embassy_time::Delay);
        // Temporary local manager just for the handshake step
        let volume_mgr = VolumeManager::new(sd_card, SdCardTimeSource);

        let _volume = volume_mgr.open_volume(VolumeIdx(0)).map_err(|_| StorageVolumeError::LowSpeedHandshakeFailed)?;
    }

    log::info!("SD CARD: Handshake verified. Shifting master clock registers to 20 MHz...");
    // TODO: sort out spi_device set_frequency for stm32
    #[cfg(any(feature = "rp2040", feature = "rp235xa", feature = "rp235xb"))]
    spi_device.bus_mut().set_frequency(20_000_000);

    // Re-mount the entire framework at full 20 MHz data rates.
    let static_spi: &'static mut SdCardSpiDevice = unsafe { core::mem::transmute(spi_device) };
    let sd_card = SdCard::new(static_spi, embassy_time::Delay);

    let volume_manager: &mut ConcreteVolumeManager =
        VOLUME_MGR_CELL.init(VolumeManager::new_with_limits(sd_card, SdCardTimeSource, 0));

    let sdcard_volume =
        volume_manager.open_volume(VolumeIdx(0)).map_err(|_| StorageVolumeError::HighSpeedReopenFailed)?;

    Ok(sdcard_volume)
}
