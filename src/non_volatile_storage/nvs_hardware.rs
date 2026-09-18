#[cfg(feature = "std")]
use embedded_storage_async::nor_flash::NorFlash;
#[cfg(feature = "std")]
use embedded_storage_file::{NorMemoryAsync, NorMemoryInFile};

#[allow(unused)]
#[cfg(feature = "storage")]
use super::nvs::{load_all_global_configs, store_all_global_configs};

#[allow(unused)]
#[cfg(any(feature = "rp2040", feature = "rp235xa", feature = "rp235xb"))]
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

#[cfg(feature = "storage")]
pub async fn load_global_configs() -> Result<(), ()> {
    #[cfg(feature = "stm32")]
    {
        //load_all_global_configs().await.map_err(|_| ())
        Ok(())
    }

    #[cfg(any(feature = "rp2040", feature = "rp235xa", feature = "rp235xb"))]
    {
        //load_all_global_configs(board_flash()).await.map_err(|_| ())
        Ok(())
    }
    #[cfg(feature = "std")]
    {
        load_all_global_configs(init_flash_driver()).await.map_err(|_| ())
    }
}

#[cfg(not(feature = "storage"))]
pub async fn load_global_configs() -> Result<(), ()> {
    core::future::ready(()).await;
    Err(())
}

// TODO: write_to_nvs to call store_global_configs
#[cfg(feature = "storage")]
pub async fn store_global_configs() -> Result<(), ()> {
    #[cfg(all(feature = "host", feature = "std"))]
    {
        let flash_driver = init_flash_driver();
        store_all_global_configs(flash_driver).await.map_err(|_| ())
    }
    #[cfg(any(feature = "rp2040", feature = "rp235xa", feature = "rp235xb"))]
    {
        Ok(())
    }
    #[cfg(feature = "stm32")]
    {
        Ok(())
    }
}

#[cfg(not(feature = "storage"))]
pub async fn store_global_configs() -> Result<(), ()> {
    core::future::ready(()).await;
    Err(())
}
