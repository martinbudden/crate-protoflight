#![doc = include_str!("README.md")]

mod config;
mod video;

#[cfg(feature = "vtx")]
pub use config::VtxConfig;
#[cfg(feature = "vtx")]
pub use video::Vtx;
