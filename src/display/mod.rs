#![allow(unused)]
#![doc = include_str!("README.md")]

mod display_port;

pub use display_port::Display;
pub use display_port::{
    DisplayPort, DisplayPortBackground, DisplayPortDeviceType, DisplayPortLayer, DisplayPortSeverity,
};
