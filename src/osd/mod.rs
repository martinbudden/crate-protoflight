#![cfg(feature = "osd")]
#![doc = include_str!("README.md")]

mod config;
mod display;
mod elements;
mod symbols;

#[allow(unused)]
pub use config::{OsdConfig, OsdElementsConfig, OsdStatsConfig};
pub use display::Osd;
//pub use elements::{OsdElement, OsdElementType, OsdElements};
