#![doc = include_str!("README.md")]

mod config;

#[cfg(feature = "gps")]
pub use config::GpsConfig;
