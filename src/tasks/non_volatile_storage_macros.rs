/// Macro to generate boilerplate non-volatile storage loader routines.
#[allow(unused)]
#[rustfmt::skip]
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
    C: CacheImpl<u16>,
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

}};}
