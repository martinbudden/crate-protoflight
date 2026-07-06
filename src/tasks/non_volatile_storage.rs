#![cfg(feature = "serde")]
#![allow(unused)]

use embedded_storage_async::nor_flash::{ErrorType, NorFlash};
use sequential_storage::{
    cache::NoCache,
    map::{MapConfig, MapStorage, PostcardValue},
};

#[cfg(feature = "rp2350")]
use embassy_rp::{
    Peri,
    flash::{Blocking, Flash},
    peripherals::FLASH,
};
#[cfg(feature = "rp2350")]
const FLASH_SIZE_BYTES: usize = 4 * 1024 * 1024;

use blackbox_logger::BlackboxConfig;
use radio_controllers::RatesConfig;
extern crate paste;

use crate::config::GLOBAL_CONFIG;
use crate::flight::ImuFilterBankConfig;
use crate::tasks::non_volatile_storage as nvs;

#[cfg(feature = "osd")]
use crate::osd::OsdConfig;
#[cfg(feature = "battery")]
use crate::sensors::BatteryConfig;

const PID_PROFILE_INDEX_KEY: u16 = 0x0001;
const RATE_PROFILE_INDEX_KEY: u16 = 0x0002;
const ACC_CALIBRATION_STATE_KEY: u16 = 0x0003;
const GYRO_CALIBRATION_STATE_KEY: u16 = 0x0004;

const MOTOR_MIXER_TYPE_KEY: u16 = 0x0005;

const ACC_OFFSET_KEY: u16 = 0x0200;
const GYRO_OFFSET_KEY: u16 = 0x0201;
const MAC_ADDRESS_KEY: u16 = 0x0202;

const DYNAMIC_NOTCH_FILTER_CONFIG_KEY: u16 = 0x0300;

// Part of PID profile
// Note that keys of items in PID profile must go up in jumps of 4, since 1 key is used for each profile
const FLIGHT_CONTROLLER_FILTERS_CONFIG_KEY: u16 = 0x0400;
const DYNAMIC_IDLE_CONTROLLER_CONFIG_KEY: u16 = 0x0404;
const FLIGHT_CONTROLLER_FLIGHTMODE_CONFIG_KEY: u16 = 0x408;
const FLIGHT_CONTROLLER_TPA_CONFIG_KEY: u16 = 0x40C;
const FLIGHT_CONTROLLER_ANTI_GRAVITY_CONFIG_KEY: u16 = 0x0410;
const FLIGHT_CONTROLLER_DMAX_CONFIG_KEY: u16 = 0x0414;
const FLIGHT_CONTROLLER_ITERM_RELAX_CONFIG_KEY: u16 = 0x0418;
const FLIGHT_CONTROLLER_YAW_SPIN_RECOVERY_CONFIG_KEY: u16 = 0x041C;
const FLIGHT_CONTROLLER_CRASH_RECOVERY_CONFIG_KEY: u16 = 0x0420;
const FLIGHT_CONTROLLER_SIMPLIFIED_PID_SETTINGS_KEY: u16 = 0x0424;
const OSD_CONFIG_KEY: u16 = 0x0428;
const OSD_ELEMENTS_CONFIG_KEY: u16 = 0x042C;

const RATES_KEY: u16 = 0x0500; // note jump of 4 to allow storage of 4 rates profiles

const IMU_FILTERS_CONFIG_KEY: u16 = 0x0600;
const RPM_FILTERS_CONFIG_KEY: u16 = 0x0601;
const FAILSAFE_CONFIG_KEY: u16 = 0x0602;
const RX_CONFIG_KEY: u16 = 0x0603;
const AUTOPILOT_CONFIG_KEY: u16 = 0x604;
const AUTOPILOT_POSITION_CONFIG_KEY: u16 = 0x605;
const ALTITUDE_HOLD_CONFIG_KEY: u16 = 0x606;
const MOTOR_CONFIG_KEY: u16 = 0x607;
const MOTOR_MIXER_CONFIG_KEY: u16 = 0x0608;
const VTX_CONFIG_KEY: u16 = 0x0609;
const GPS_CONFIG_KEY: u16 = 0x060A;
const FLIGHT_CONTROLLER_CRASH_FLIP_KEY: u16 = 0x060B;
const RC_MODES_ACTIVATION_CONDITIONS_KEY: u16 = 0x060C;
const RC_ADJUSTMENT_RANGES_KEY: u16 = 0x060D;
const FEATURES_CONFIG_KEY: u16 = 0x060E;
const BLACKBOX_CONFIG_KEY: u16 = 0x060F;
const BATTERY_CONFIG_KEY: u16 = 0x0610;

/// Macro to generate boilerplate non-volatile storage loader routines.
macro_rules! generate_config_handlers {
    ($prefix:ident, $key:expr, $buf_size:expr) => {
        paste::paste! {
            // 1. Configure the PostcardValue macro for the Option wrap variant
            //impl<'a> PostcardValue<'a> for Option<[<$prefix Config>]> {}

            // 2. Generate the LOAD function
            pub async fn [<load_ $prefix:lower _config>]<F>(
                config: &mut [<$prefix Config>],
                storage: &mut MapStorage<u16, F, NoCache>
            ) where F: NorFlash {
                let mut buffer = [0u8; $buf_size];

                if let Ok(Some(Some(loaded_data))) = storage
                    .fetch_item::<Option<[<$prefix Config>]>>(&mut buffer, &$key)
                    .await
                {
                    *config = loaded_data;
                }
            }

            // 3. Generate the SAVE function
            pub async fn [<save_ $prefix:lower _config>]<F>(
                config: &[<$prefix Config>],
                storage: &mut MapStorage<u16, F, NoCache>
            ) -> Result<(), sequential_storage::Error<<F as ErrorType>::Error>>
            where
                F: NorFlash,
                [<$prefix Config>]: PartialEq // Enforces that the struct derives PartialEq
            {
                let mut buffer = [0u8; $buf_size];

                // 1. READ BEFORE WRITE: Fetch the current active item from flash
                if let Ok(Some(Some(existing_data))) = storage
                    .fetch_item::<Option<[<$prefix Config>]>>(&mut buffer, &$key)
                    .await
                {
                    // 2. CHECK EQUALITY: If identical, skip the write entirely
                    if &existing_data == config {
                        #[cfg(not(target_arch = "arm"))]
                        //println!("[NVS]: Data matches flash exactly. Skipping redundant write.");
                        return Ok(());
                    }
                }

                // 3. WRITE ONLY IF CHANGED: Clear or reuse the buffer for serialization
                let data_to_save = Some(config.clone());
                storage.store_item(&mut buffer, &$key, &data_to_save).await
            }

            // 4. Generate the DELETE function
            pub async fn [<delete_ $prefix:lower _config>]<F>(
                storage: &mut MapStorage<u16, F, NoCache>
            ) -> Result<(), sequential_storage::Error<<F as ErrorType>::Error>>
            where F: NorFlash {
                let mut buffer = [0u8; $buf_size];

                // 1. READ BEFORE DELETE: Check what is currently stored under this key
                // .fetch_item returns Ok(Some(StoredValue)) if the key is found.
                // StoredValue itself is an Option<Config>. If it is already `None`, it's deleted.
                if let Ok(Some(None)) = storage
                    .fetch_item::<Option<[<$prefix Config>]>>(&mut buffer, &$key)
                    .await
                {
                    // 2. CHECK STATUS: If a None marker is already active, skip the write
                    //#[cfg(not(target_arch = "arm"))]
                    //println!("[NVS]: Subsystem is already deleted in flash. Skipping redundant marker write.");
                    return Ok(());
                }

                // 3. WRITE ONLY IF NOT ALREADY DELETED
                let delete_marker: Option<[<$prefix Config>]> = None;
                storage.store_item(&mut buffer, &$key, &delete_marker).await
            }
        }
    };
}
#[cfg(feature = "osd")]
generate_config_handlers!(Osd, OSD_CONFIG_KEY, 128);
#[cfg(feature = "blackbox")]
generate_config_handlers!(Blackbox, BLACKBOX_CONFIG_KEY, 128);
generate_config_handlers!(Rates, RATES_KEY, 128);

pub async fn save_all_system_configs<F>(flash: &mut F, flash_range: core::ops::Range<u32>)
where
    F: NorFlash,
{
    // 1. Establish the driver handle matching your u16 Key setup
    let mut storage = MapStorage::new(flash, MapConfig::new(flash_range), NoCache::new());

    // 2. Obtain an immutable lock on the configuration data structures
    let config = GLOBAL_CONFIG.lock().await;

    // 3. Save values via your clean `lds` namespace shortcut
    _ = save_imu_filter_bank_config(&config.imu_filter_bank, &mut storage).await;
    //let _ = save_rates_config(&config.rates, &mut storage).await;
    //let _ = save_blackbox_config(&config.blackbox, &mut storage).await;

    //println!("[NVS]: All modified configurations saved successfully!");
}

// Update the functions to receive `&mut MapStorage` directly
pub async fn load_imu_filter_bank_config<F>(config: &mut ImuFilterBankConfig, storage: &mut MapStorage<u16, F, NoCache>)
where
    F: NorFlash,
{
    let mut buffer = [0u8; 256];

    // Directly execute the item lookup method on your driver storage object
    if let Ok(Some(loaded_data)) = storage.fetch_item(&mut buffer, &IMU_FILTERS_CONFIG_KEY).await {
        *config = loaded_data;
    }
}
pub async fn save_imu_filter_bank_config<F>(
    config: &ImuFilterBankConfig, // Passed as an immutable reference to read from
    storage: &mut MapStorage<u16, F, NoCache>,
) -> Result<(), sequential_storage::Error<<F as ErrorType>::Error>>
// Returns a Result to verify success
where
    F: NorFlash,
{
    let mut buffer = [0u8; 256]; // Serialization workspace

    // Appends the updated struct to flash under the unique key
    storage.store_item(&mut buffer, &IMU_FILTERS_CONFIG_KEY, config).await
}

pub async fn delete_imu_filter_bank_config<F>(
    storage: &mut MapStorage<u16, F, NoCache>,
) -> Result<(), sequential_storage::Error<<F as ErrorType>::Error>>
where
    F: NorFlash,
{
    let mut buffer = [0u8; 256];

    // Appends `None` to flash. This acts as a deletion marker.
    let delete_marker: Option<ImuFilterBankConfig> = None;
    storage.store_item(&mut buffer, &IMU_FILTERS_CONFIG_KEY, &delete_marker).await
}

// ==========================================
// 1. LOADING PATTERN (Unwraps internal Option)
// ==========================================

#[cfg(feature = "battery")]
pub async fn load_battery_config<F>(
    config: &mut BatteryConfig,
    storage: &mut MapStorage<u16, F, NoCache>,
) where
    F: NorFlash,
{
    let mut buffer = [0u8; 256];

    // Fetch as an Option, but unwrap it back into your default struct
    if let Ok(Some(Some(loaded_data))) =
        storage.fetch_item::<Option<BatteryConfig>>(&mut buffer, &BATTERY_CONFIG_KEY).await
    {
        *config = loaded_data;
    }
}

// ==========================================
// 2. SAVING PATTERN (Wraps with Some right before flash write)
// ==========================================
#[cfg(feature = "battery")]
pub async fn save_battery_config<F>(
    config: &BatteryConfig,
    storage: &mut MapStorage<u16, F, NoCache>,
) -> Result<(), sequential_storage::Error<<F as ErrorType>::Error>>
where
    F: NorFlash,
{
    let mut buffer = [0u8; 256];
    if *config == BatteryConfig::default() {
        // Appends `None` to flash. This acts as a deletion marker.
        let delete_marker: Option<BatteryConfig> = None;
        storage.store_item(&mut buffer, &BATTERY_CONFIG_KEY, &delete_marker).await
    } else {
        // Wraps raw struct into an option wrapper for the wear-level metadata tracking
        storage.store_item(&mut buffer, &BATTERY_CONFIG_KEY, &Some(*config)).await
    }
}

// ==========================================
// 3. DELETION PATTERN (Appends a raw None marker to flash)
// ==========================================
#[cfg(feature = "battery")]
pub async fn delete_battery_config<F>(
    storage: &mut MapStorage<u16, F, NoCache>,
) -> Result<(), sequential_storage::Error<<F as ErrorType>::Error>>
where
    F: NorFlash,
{
    let mut buffer = [0u8; 256];

    // Appends `None` to flash. This acts as a deletion marker.
    let delete_marker: Option<BatteryConfig> = None;
    // Note store_item writes a new item even if
    storage.store_item(&mut buffer, &BATTERY_CONFIG_KEY, &delete_marker).await
}

/*pub async fn load_rates_config<F>(
    config: &mut RatesConfig,
    _storage: &mut MapStorage<u16, F, NoCache>
) where
    F: NorFlash
{
    *config = RatesConfig::default();
    /*let mut buffer = [0u8; 64];
    if let Ok(Some(loaded_data)) = storage.fetch_item(&mut buffer, &RATES_KEY).await {
        *config = loaded_data;
    }*/
}

pub async fn load_blackbox_config<F>(
    config: &mut BlackboxConfig,
    _storage: &mut MapStorage<u16, F, NoCache>
) where
    F: NorFlash
{
    *config = BlackboxConfig::default();
    /*let mut buffer = [0u8; 256];
    if let Ok(Some(loaded_data)) = storage.fetch_item(&mut buffer, &BLACKBOX_KEY).await {
        *config = loaded_data;
    }*/
}*/

// --- 1. PC (Host) Build Configuration ---
// If building on your PC (x86_64, Mac, etc.)
// FIX: Replace `_` with `impl embedded_storage_async::nor_flash::NorFlash`
#[cfg(feature = "std")]
pub fn init_flash_driver() -> impl embedded_storage_async::nor_flash::NorFlash {
    use embedded_storage_file::{NorMemoryAsync, NorMemoryInFile};

    let path = "pc_mock_flash.nor";
    let capacity_bytes = 1024 * 1024; // Allocate a 1MB virtual flash file

    // 1. Instantiate the synchronous inner file backend with layout properties:
    //    <READ_SIZE, WRITE_SIZE, ERASE_SIZE>
    #[allow(clippy::expect_used)]
    let inner_sync_nor = NorMemoryInFile::<256, 256, 4096>::new(path, capacity_bytes)
        .expect("Failed to create synchronous mock flash file");

    // 2. FIX: Wrap it using the single-parameter asynchronous wrapper.
    //    We remove the <256, 256, 4096> from NorMemoryAsync to satisfy the 1-generic rule.
    NorMemoryAsync::new(inner_sync_nor)
}

#[cfg(not(feature = "std"))]
pub fn init_flash_driver<'d>(
    // Pass the Peri structural instance bound to the FLASH singleton type
    flash_pin: Peri<'d, FLASH>,
) -> Flash<'d, FLASH, Blocking, FLASH_SIZE_BYTES> {
    Flash::<_, Blocking, FLASH_SIZE_BYTES>::new_blocking(flash_pin)
}

/*pub async fn load_global_configs<F>(flash: &mut F, flash_range: core::ops::Range<u32>)
where
    F: NorFlash,
{
    // Initialize the modern storage driver handle matching your u16 Key setup
    //let mut storage = MapStorage::new(flash, MapConfig::new(flash_range), NoCache::new());

    let mut config = GLOBAL_CONFIG.lock().await;
    //nvs::load_imu_filter_bank_config(&mut config.imu_filter_bank, &mut storage).await;
}*/

#[cfg(feature = "std")]
pub async fn load_global_configs() {
     // Full 1MB simulated range for PC tests
    /*let flash_range = 0..1024 * 1024;
    // Initialize our conditional target driver
    let mut flash_driver = init_flash_driver();
    let mut storage = MapStorage::new(flash_driver, MapConfig::new(flash_range), NoCache::new());*/

    let mut config = GLOBAL_CONFIG.lock().await;
    //nvs::load_imu_filter_bank_config(&mut config.imu_filter_bank, &mut storage).await;
    //nvs::load_rates_config(&mut config.rates, &mut flash_driver, config_flash_range.clone());
}

// Standard Raspberry Pi Pico 2 boards have 4MB of onboard QSPI flash memory.
#[cfg(feature = "rp2350")]
pub async fn load_global_configs() {
    let flash_range = (4096 - 128) * 1024 .. 4096 * 1024; // Tail end 128KB for chip
    let mut flash_driver = {
        let p = embassy_rp::init(Default::default());
        init_flash_driver(p)
    };
    let mut storage = MapStorage::new(flash, MapConfig::new(flash_range), NoCache::new());
    let mut config = GLOBAL_CONFIG.lock().await;
    nvs::load_imu_filter_bank_config(&mut config.imu_filter_bank, &mut storage).await;
}