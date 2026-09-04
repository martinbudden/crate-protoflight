#![cfg(feature = "serde")]

#[cfg(feature = "std")]
use embedded_storage_async::nor_flash::NorFlash;
#[cfg(feature = "std")]
use embedded_storage_file::{NorMemoryAsync, NorMemoryInFile};

use super::nvs::{load_all_global_configs, store_all_global_configs};

#[cfg(feature = "rp2350")]
use {
    embassy_embedded_hal::adapter::BlockingAsync,
    embassy_rp::{
        Peri,
        flash::{Blocking, Flash},
        peripherals::FLASH,
    },
};

// PC (Host) Build Configuration --- If building on your PC (x86_64, Mac, etc)
#[cfg(feature = "std")]
pub fn init_flash_driver() -> impl NorFlash {
    let path = "pc_mock_flash.nor";
    let capacity_bytes = 1024 * 1024; // 1MB 

    #[allow(clippy::expect_used)]
    let inner_sync_nor =
        NorMemoryInFile::<4, 4, 4096>::new(path, capacity_bytes).expect("Failed to create synchronous mock flash file");

    NorMemoryAsync::new(inner_sync_nor)
}

#[cfg(not(feature = "std"))]
pub fn init_flash_driver() {}

pub async fn load_global_configs() -> Result<(), ()> {
    #[cfg(feature = "rp2350")]
    {
        load_global_configs(board_flash()).await.map_err(|_| ())
    }
    #[cfg(feature = "std")]
    {
        load_all_global_configs(init_flash_driver()).await.map_err(|_| ())
    }
}

// TODO: write_to_nvs to call store_global_configs
pub async fn store_global_configs() -> Result<(), ()> {
    #[cfg(all(feature = "host", feature = "std"))]
    {
        let flash_driver = init_flash_driver();
        store_all_global_configs(flash_driver).await.map_err(|_| ())
    }
}
