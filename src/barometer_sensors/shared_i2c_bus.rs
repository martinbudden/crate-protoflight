use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

// Bus = raw hardware peripheral
// Device = bus + chip select + transaction locking

#[cfg(feature = "host")]
pub type I2cDeviceBlocking = crate::i2c_bus::MockI2c;

#[cfg(feature = "esp32s3")]
pub type I2cDeviceBlocking = crate::i2c_bus::MockI2c;

#[cfg(any(feature = "rp2040", feature = "rp235xa", feature = "rp235xb"))]
pub type I2cDeviceBlocking = embassy_rp::i2c::I2c<'static, embassy_rp::peripherals::I2C0, embassy_rp::i2c::Blocking>;

#[cfg(feature = "stm32")]
pub type I2cDeviceBlocking =
    embassy_stm32::i2c::I2c<'static, embassy_stm32::mode::Blocking, embassy_stm32::i2c::mode::Master>;

pub type SharedI2cBus = Mutex<NoopRawMutex, I2cDeviceBlocking>;
