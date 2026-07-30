#![cfg(feature = "osd")]
#![doc = include_str!("README.md")]

mod config;
mod display;
mod elements;
mod elements_draw;
mod fixed_buf;
mod osd_buffer_cursor;
mod symbols;

#[allow(unused)]
pub use config::{OsdConfig, OsdElementsConfig, OsdStatsConfig, PilotConfig};
pub use display::{Osd, OsdDrawContext};
//pub use elements::{OsdElement, OsdElementType, OsdElements};
