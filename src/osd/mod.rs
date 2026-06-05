#![cfg(feature = "osd")]
#![doc = include_str!("README.md")]

mod config;
mod display;
mod elements;
mod elements_draw;
mod symbols;

#[allow(unused)]
pub use config::{OsdConfig, OsdElementsConfig, OsdStatsConfig};
pub use display::{Osd, OsdDrawContext};
//pub use elements::{OsdElement, OsdElementType, OsdElements};
