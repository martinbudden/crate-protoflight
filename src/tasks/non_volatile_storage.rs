#![cfg(feature = "serde")]

use embedded_storage_async::nor_flash::NorFlash;

#[allow(unused)]
use sequential_storage::{
    cache::{Cache, CacheImpl},
    map::{MapConfig, MapStorage},
};

#[cfg(feature = "std")]
use embedded_storage_file::{NorMemoryAsync, NorMemoryInFile};

#[cfg(feature = "rp2350")]
use {
    embassy_embedded_hal::adapter::BlockingAsync,
    embassy_rp::{
        Peri,
        flash::{Blocking, Flash},
        peripherals::FLASH,
    },
};

#[allow(unused)]
#[cfg(feature = "rp2350")]
const FLASH_SIZE_BYTES: usize = 4 * 1024 * 1024;
#[allow(unused)]
#[cfg(not(feature = "rp2350"))]
const FLASH_SIZE_BYTES: u32 = 4 * 1024 * 1024;

extern crate paste;

struct Key {}

#[allow(unused)]
impl Key {
    const PID_PROFILE_INDEX: u16 = 0x0001;
    const RATE_PROFILE_INDEX: u16 = 0x0002;
    const ACC_CALIBRATION_STATE: u16 = 0x0003;
    const GYRO_CALIBRATION_STATE: u16 = 0x0004;

    const MOTOR_MIXER_TYPE: u16 = 0x0005;

    const ACC_OFFSET: u16 = 0x0200;
    const GYRO_OFFSET: u16 = 0x0201;
    const MAC_ADDRESS: u16 = 0x0202;

    const DYNAMIC_NOTCH_FILTER_CONFIG: u16 = 0x0300;

    // Part of PID profile
    // Note that keys of items in PID profile must go up in jumps of 4, since 1 key is used for each profile
    const FLIGHT_CONTROLLER_FILTERS_CONFIG: u16 = 0x0400;
    const DYNAMIC_IDLE_CONTROLLER_CONFIG: u16 = 0x0404;
    const FLIGHT_CONTROLLER_FLIGHTMODE_CONFIG: u16 = 0x408;
    const FLIGHT_CONTROLLER_TPA_CONFIG: u16 = 0x40C;
    const FLIGHT_CONTROLLER_ANTI_GRAVITY_CONFIG: u16 = 0x0410;
    const FLIGHT_CONTROLLER_DMAX_CONFIG: u16 = 0x0414;
    const FLIGHT_CONTROLLER_ITERM_RELAX_CONFIG: u16 = 0x0418;
    const FLIGHT_CONTROLLER_YAW_SPIN_RECOVERY_CONFIG: u16 = 0x041C;
    const FLIGHT_CONTROLLER_CRASH_RECOVERY_CONFIG: u16 = 0x0420;
    const FLIGHT_CONTROLLER_SIMPLIFIED_PID_SETTINGS: u16 = 0x0424;
    const OSD_CONFIG: u16 = 0x0428;
    const OSD_ELEMENTS_CONFIG: u16 = 0x042C;

    const RATES: u16 = 0x0500; // note jump of 4 to allow storage of 4 rates profiles

    const IMU_FILTERS_CONFIG: u16 = 0x0600;
    const RPM_FILTERS_CONFIG: u16 = 0x0601;
    const FAILSAFE_CONFIG: u16 = 0x0602;
    const RX_CONFIG: u16 = 0x0603;
    const AUTOPILOT_CONFIG: u16 = 0x604;
    const AUTOPILOT_POSITION_CONFIG: u16 = 0x605;
    const ALTITUDE_HOLD_CONFIG: u16 = 0x606;
    const MOTOR_CONFIG: u16 = 0x607;
    const MOTOR_MIXER_CONFIG: u16 = 0x0608;
    const VTX_CONFIG: u16 = 0x0609;
    const GPS_CONFIG: u16 = 0x060A;
    const FLIGHT_CONTROLLER_CRASH_FLIP: u16 = 0x060B;
    const RC_MODES_ACTIVATION_CONDITIONS: u16 = 0x060C;
    const RC_ADJUSTMENT_RANGES: u16 = 0x060D;
    const FEATURES_CONFIG: u16 = 0x060E;
    const BLACKBOX_CONFIG: u16 = 0x060F;
    const BATTERY_CONFIG: u16 = 0x0610;
    const ARMING_CONFIG: u16 = 0x0611;
}
/*
There are two layers of `Option`

fetch_item
    │
    ├── None
    │     → key doesn't exist
    │
    └── Some(...)
          │
          ├── None
          │     → record has been deleted
          │
          └── Some(ArmingConfig)
                → actual configuration
*/

/// Macro to generate boilerplate non-volatile storage loader routines.
macro_rules! generate_config_handlers {
    ($location:expr, $prefix:ident, $key:expr, $buf_size:expr) => {
        paste::paste! {
            // 1. Configure the PostcardValue macro for the Option wrap variant
            //impl<'a> PostcardValue<'a> for Option<[<$prefix Config>]> {}

            use $location::[<$prefix Config>];

            // Generate the LOAD function
            #[allow(unused)]
            pub async fn [<load_ $prefix:lower _config>]<F, C>(
                config: &mut [<$prefix Config>],
                storage: &mut MapStorage<u16, F, C>,
            ) -> Result<(), sequential_storage::Error<F::Error>>
            where
                F: NorFlash,
                C: CacheImpl<u16>
            {
                let mut buffer = [0u8; $buf_size];
                let stored = storage.fetch_item::<Option<[<$prefix Config>]>>(&mut buffer, &$key).await?;
                match stored {
                    Some(Some(loaded_data)) => {
                        *config = loaded_data;
                    }
                    None | Some(None) => {
                        *config = [<$prefix Config>]::default();
                    }
                }
                Ok(())
            }

            // Generate the SAVE function
            #[allow(unused)]
            pub async fn [<save_ $prefix:lower _config>]<F, C>(
                config: &[<$prefix Config>],
                storage: &mut MapStorage<u16, F, C>
            ) -> Result<(), sequential_storage::Error<F::Error>>
            where
                F: NorFlash,
                C: CacheImpl<u16>,
                [<$prefix Config>]: PartialEq // Enforces that the struct derives PartialEq
            {
                let mut buffer = [0u8; $buf_size];
                // READ BEFORE SAVE: Check what is currently stored under this key
                let stored = storage.fetch_item::<Option<[<$prefix Config>]>>(&mut buffer, &$key).await?;
                if *config == [<$prefix Config>]::default() {
                    // Default configuration means "no stored configuration".
                    // If there is an existing configuration, append a None marker to mark it deleted.
                    // If there is no record, do nothing.
                    if matches!(stored, Some(Some(_))) {
                        let delete_marker: Option<ArmingConfig> = None;
                        storage.store_item(&mut buffer, &$key, &delete_marker).await?;
                    }
                } else {
                    // Non-default configuration:
                    // only write it if it differs from the currently stored value.
                    if !matches!(stored, Some(Some(stored_config)) if stored_config == *config) {
                        storage.store_item(&mut buffer, &$key, &Some(*config)).await?;
                    }
                }
                Ok(())
            }

            // Generate the DELETE function
            #[allow(unused)]
            pub async fn [<delete_ $prefix:lower _config>]<F, C>(
                storage: &mut MapStorage<u16, F, C>
            ) -> Result<(), sequential_storage::Error<F::Error>>
            where
                F: NorFlash,
                C: CacheImpl<u16>
            {
                let mut buffer = [0u8; $buf_size];
                // READ BEFORE DELETE: Check what is currently stored under this key
                let stored =storage.fetch_item::<Option<[<$prefix Config>]>>(&mut buffer, &$key).await?;
                // WRITE ONLY IF NOT ALREADY DELETED
                if matches!(stored, Some(Some(_))) {
                    let delete_marker: Option<[<$prefix Config>]> = None;
                    storage.store_item(&mut buffer, &$key, &delete_marker).await?;
                }
                Ok(())
            }
        }
    };
}

generate_config_handlers!(radio_controllers, Rates, Key::RATES, 256);

#[cfg(feature = "osd")]
generate_config_handlers!(crate::osd, Osd, Key::OSD_CONFIG, 256);

#[cfg(feature = "blackbox")]
generate_config_handlers!(blackbox_logger, Blackbox, Key::BLACKBOX_CONFIG, 256);

#[cfg(feature = "battery")]
generate_config_handlers!(crate::sensors, Battery, Key::BATTERY_CONFIG, 256);

use crate::flight::ArmingConfig;

/// Load from NVS (Unwraps `Option`).
pub async fn load_arming_config<F, C>(
    config: &mut ArmingConfig,
    storage: &mut MapStorage<u16, F, C>,
) -> Result<(), sequential_storage::Error<F::Error>>
where
    F: NorFlash,
    C: CacheImpl<u16>,
{
    let mut buffer = [0u8; 256];
    let stored = storage.fetch_item::<Option<ArmingConfig>>(&mut buffer, &Key::ARMING_CONFIG).await?;
    match stored {
        Some(Some(loaded_data)) => {
            *config = loaded_data;
        }
        None | Some(None) => {
            *config = ArmingConfig::default();
        }
    }
    Ok(())
}

/// Wrap with `Some` and store to NVS.
/*
| Existing flash    | `config`    | Action               |
| ----------------- | ----------- | -------------------- |
| No record         | `default()` | Nothing              |
| `None` marker     | `default()` | Nothing              |
| `Some(old)`       | `default()` | Store `None`         |
| No record         | non-default | Store `Some(config)` |
| `None` marker     | non-default | Store `Some(config)` |
| `Some(same)`      | non-default | Nothing              |
| `Some(different)` | non-default | Store `Some(config)` |

None
    → no record

Some(None)
    → record exists, but represents deleted/default

Some(Some(config))
    → actual stored configuration
*/
#[allow(unused)]
pub async fn save_arming_config<F, C>(
    config: &ArmingConfig,
    storage: &mut MapStorage<u16, F, C>,
) -> Result<(), sequential_storage::Error<F::Error>>
where
    F: NorFlash,
    C: CacheImpl<u16>,
{
    let mut buffer = [0u8; 256];
    let stored = storage.fetch_item::<Option<ArmingConfig>>(&mut buffer, &Key::ARMING_CONFIG).await?;
    if *config == ArmingConfig::default() {
        // Default configuration means "no stored configuration".
        // If there is an existing configuration, append a None marker
        // to mark it deleted. If there is no record, do nothing.
        if matches!(stored, Some(Some(_))) {
            let delete_marker: Option<ArmingConfig> = None;
            storage.store_item(&mut buffer, &Key::ARMING_CONFIG, &delete_marker).await?;
        }
    } else {
        // Non-default configuration:
        // only write it if it differs from the currently stored value.
        if !matches!(stored, Some(Some(stored_config)) if stored_config == *config) {
            storage.store_item(&mut buffer, &Key::ARMING_CONFIG, &Some(*config)).await?;
        }
    }

    Ok(())
}
/// Delete by appending a `None` marker to NVS.
/*
fetch
 │
 ├── None           → nothing to delete
 │
 ├── Some(None)     → already deleted
 │
 └── Some(Some(_))  → append deletion marker

 flash error → return error
*/
#[allow(unused)]
pub async fn delete_arming_config<F, C>(
    storage: &mut MapStorage<u16, F, C>,
) -> Result<(), sequential_storage::Error<F::Error>>
where
    F: NorFlash,
    C: CacheImpl<u16>,
{
    let mut buffer = [0u8; 256];

    // READ BEFORE DELETE: Check what is currently stored under this key
    let stored = storage.fetch_item::<Option<ArmingConfig>>(&mut buffer, &Key::ARMING_CONFIG).await?;

    // WRITE ONLY IF NOT ALREADY DELETED
    if matches!(stored, Some(Some(_))) {
        let delete_marker: Option<ArmingConfig> = None;
        storage.store_item(&mut buffer, &Key::ARMING_CONFIG, &delete_marker).await?;
    }

    Ok(())
}

// PC (Host) Build Configuration --- If building on your PC (x86_64, Mac, etc.)
#[cfg(feature = "std")]
pub fn init_flash_driver() -> impl NorFlash {
    let path = "pc_mock_flash.nor";
    let capacity_bytes = 1024 * 1024; // 1MB 

    #[allow(clippy::expect_used)]
    let inner_sync_nor =
        NorMemoryInFile::<4, 4, 4096>::new(path, capacity_bytes).expect("Failed to create synchronous mock flash file");

    NorMemoryAsync::new(inner_sync_nor)
}

//#[cfg(feature = "std")]
pub async fn load_global_configs<F>(flash_driver: F) -> Result<(), sequential_storage::Error<F::Error>>
where
    F: NorFlash,
{
    use crate::{config::GLOBAL_CONFIG, tasks::non_volatile_storage as nvs};

    let map_config = MapConfig::new(0..FLASH_SIZE_BYTES);
    let cache = Cache::new_uncached();
    let mut map_storage = MapStorage::new(flash_driver, map_config, cache);

    let mut config = GLOBAL_CONFIG.lock().await;

    nvs::load_arming_config(&mut config.arming, &mut map_storage).await?;

    Ok(())
}
#[cfg(all(test, feature = "std"))]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;

    use crate::tasks::non_volatile_storage::{load_arming_config, save_arming_config};
    /*
    No record
        │
        ├── load ──────────────► default
        │
        └── save(config A)
                  │
                  ▼
             config A
                  │
                  ├── load ─────► config A
                  │
                  ├── save(A) ──► config A       (no change)
                  │
                  └── save(default)
                             │
                             ▼
                          deleted
                             │
                             ├── load ─────► default
                             │
                             └── save(default) → deleted (no change)

                          deleted
                             │
                             └── save(config B)
                                       │
                                       ▼
                                    config B
    */
    #[test]
    fn test_arming_config_save_and_reload() {
        /*
        empty flash
            │
            ├── load ───────────────► default
            │
            ├── save test_config
            │
            ├── load ───────────────► test_config
            │
            ├── save test_config again
            │
            └── load ───────────────► test_config

        Some(test_config)
               │
               │ save(default)
               ▼
           Some(None)
               │
               │ load
               ▼
             default
        */
        futures::executor::block_on(async {
            let path = "test_arming_config.nor";

            // Start with a clean mock flash.
            let _ = std::fs::remove_file(path);

            let capacity_bytes = 1024 * 1024;

            #[allow(clippy::expect_used)]
            let inner_sync_nor =
                NorMemoryInFile::<4, 4, 4096>::new(path, capacity_bytes).expect("Failed to create test mock flash");

            let flash_driver = NorMemoryAsync::new(inner_sync_nor);

            #[allow(clippy::cast_possible_truncation)]
            let map_config = MapConfig::new(0..capacity_bytes as u32);
            let cache = Cache::new_uncached();

            let mut storage = MapStorage::new(flash_driver, map_config, cache);

            // Initially there should be no stored configuration,
            // so loading should produce the default.
            let default_config = ArmingConfig::default();
            let mut config = ArmingConfig::default();

            load_arming_config(&mut config, &mut storage).await.expect("Failed to load initial arming config");

            assert_eq!(config, ArmingConfig::default());

            // Create a deliberately non-default configuration.
            let test_config =
                ArmingConfig { gyro_calibrate_on_first_arm: 1, auto_disarm_delay: 50, prearm_allow_rearm: 1 };
            assert_ne!(default_config, test_config);

            // Save it.
            save_arming_config(&test_config, &mut storage).await.expect("Failed to save arming config");

            // Load it back.
            let mut loaded_config = ArmingConfig::default();

            load_arming_config(&mut loaded_config, &mut storage).await.expect("Failed to reload arming config");

            // Verify the round trip.
            assert_eq!(loaded_config, test_config);

            // Save the same configuration again.
            save_arming_config(&test_config, &mut storage).await.expect("Failed to save arming config");

            // Load it again.
            let mut loaded_config = ArmingConfig::default();

            load_arming_config(&mut loaded_config, &mut storage).await.expect("Failed to reload arming config");

            assert_eq!(loaded_config, test_config);

            // Saving the default configuration should delete the stored configuration.
            let default_config = ArmingConfig::default();

            save_arming_config(&default_config, &mut storage).await.expect("Failed to save default arming config");

            // Loading after deletion should return the default.
            let mut loaded_config = ArmingConfig::default();

            load_arming_config(&mut loaded_config, &mut storage)
                .await
                .expect("Failed to load arming config after deletion");

            assert_eq!(loaded_config, ArmingConfig::default());

            // Saving the default configuration again should do nothing.
            save_arming_config(&default_config, &mut storage).await.expect("Failed to save default arming config");

            // It should still load as the default configuration.
            let mut loaded_config = ArmingConfig::default();

            load_arming_config(&mut loaded_config, &mut storage).await.expect("Failed to load arming config");

            assert_eq!(loaded_config, ArmingConfig::default());

            // Saving a new non-default configuration after deletion should store it.
            let new_test_config =
                ArmingConfig { gyro_calibrate_on_first_arm: 1, auto_disarm_delay: 20, prearm_allow_rearm: 0 };

            save_arming_config(&new_test_config, &mut storage).await.expect("Failed to save new arming config");

            // Loading should now return the new configuration.
            let mut loaded_config = ArmingConfig::default();

            load_arming_config(&mut loaded_config, &mut storage).await.expect("Failed to load new arming config");

            assert_eq!(loaded_config, new_test_config);

            // Clean up the test flash file.
            drop(storage);

            let _ = std::fs::remove_file(path);
        });
    }
}
