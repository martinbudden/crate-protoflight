mod config;
mod magnetometer;
mod magnetometer_message;

pub use config::MagnetometerConfig;
pub use magnetometer::{Magnetometer, MagnetometerType};
pub use magnetometer_message::MagnetometerMessage;
