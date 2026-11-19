#![allow(unused)]
mod magnetometer_common;
mod magnetometer_device;

mod magnetometer_hmc5883;
mod magnetometer_mock;

pub use magnetometer_common::MagnetometerCommon;
pub use magnetometer_device::{MagnetometerDevice, MagnetometerI2cError};

pub use magnetometer_hmc5883::MagnetometerHmc5883;
pub use magnetometer_mock::MagnetometerMock;
