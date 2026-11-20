mod config;
mod magnetometer;
mod magnetometer_message;

pub use magnetometer::{Magnetometer, MagnetometerType};
pub use magnetometer_message::MagnetometerMessage;

#[cfg(feature = "magnetometer")]
pub use config::MagnetometerConfig;
