mod airb_omnibus_f4;
mod airb_omnibus_f4_sd;
mod host;
mod madflight_fc3;
mod rpi_pico2;
mod sp_racing_f4_evo;
mod speedybee_f405_v4;

#[cfg(feature = "host")]
pub use host::{Board, BoardImu};

#[cfg(feature = "rpi_pico2")]
pub use rpi_pico2::{Board, BoardImu};

#[cfg(feature = "madflight_fc3")]
pub use madflight_fc3::{Board, BoardImu};

#[cfg(feature = "speedybee_f405_v4")]
pub use speedybee_f405_v4::{Board, BoardImu};

#[cfg(feature = "sp_racing_f4_evo")]
pub use sp_racing_f4_evo::{Board, BoardImu};

#[cfg(feature = "airb_omnibus_f4")]
pub use airb_omnibus_f4::{Board, BoardImu};

#[cfg(feature = "airb_omnibus_f4_sd")]
pub use airb_omnibus_f4_sd::{Board, BoardImu};
