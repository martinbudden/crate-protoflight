//#![allow(clippy::doc_markdown)]
#![doc = include_str!("README.md")]

// Macros must be brought into scope before the modules that use them.

#[macro_use]
mod macros;

mod nvs;

#[cfg(feature = "serde")]
pub use nvs::{init_flash_driver, load_global_configs};
