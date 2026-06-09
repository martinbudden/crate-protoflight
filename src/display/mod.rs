#![doc = include_str!("README.md")]

mod display_port;
mod display_port_max7456;
mod display_port_mock;
mod display_port_msp;

pub use display_port::Display;
#[allow(unused)]
pub use display_port::{
    DisplayClear, DisplayPort, DisplayPortBackground, DisplayPortDeviceType, DisplayPortLayer, DisplayPortSeverity,
};

#[allow(unused)]
pub use display_port_max7456::DisplayPortMax7456;
#[allow(unused)]
pub use display_port_mock::DisplayPortMock;
