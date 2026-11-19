mod barometer_common;
mod barometer_device;

mod barometer_bmp280;
mod barometer_dps310;
mod barometer_mock;

pub use barometer_common::BarometerCommon;
pub use barometer_device::{BarometerI2cError, SharedI2cBus};

pub use barometer_bmp280::BarometerBmp280;
pub use barometer_device::BarometerDevice;
pub use barometer_dps310::BarometerDps310;
pub use barometer_mock::BarometerMock;
