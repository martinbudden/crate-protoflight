#![doc = include_str!("README.md")]
#![allow(unused)]

mod config;
mod rangefinder;
mod rangefinder_hcsr04;
mod rangefinder_mock;

pub use config::RangefinderConfig;
pub use rangefinder::{Rangefinder, RangefinderMessage, RangefinderType, RxRangefinder};
