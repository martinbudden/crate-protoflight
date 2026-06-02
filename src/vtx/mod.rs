#![cfg(feature = "vtx")]
#![doc = include_str!("README.md")]

mod config;
mod video;

pub use config::VtxConfig;
pub use video::Vtx;
