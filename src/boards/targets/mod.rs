mod airb_omnibus_f4;
mod airb_omnibus_f4_sd;
mod host;
mod madflight_fc2;
mod madflight_fc3;
mod openfc_lite;
mod openfc_lite_mini;
mod rpi_pico2;
mod sp_racing_f4_evo;
mod speedybee_f405_v4;
mod stamp_s3_fly;

#[cfg(feature = "host")]
pub use host::{Board, BoardImu};

#[allow(unused)]
#[cfg(feature = "rpi_pico2")]
pub use rpi_pico2::{Board, BoardImu, SdCardSpiDevice};

#[allow(unused)]
#[cfg(feature = "madflight_fc2")]
pub use madflight_fc2::{Board, BoardImu, SdCardSpiDevice};

#[cfg(feature = "madflight_fc3")]
pub use madflight_fc3::{Board, BoardImu, SdCardSpiDevice};

#[cfg(feature = "speedybee_f405_v4")]
pub use speedybee_f405_v4::{Board, BoardImu, SdCardSpiDevice};

#[cfg(feature = "sp_racing_f4_evo")]
pub use sp_racing_f4_evo::{Board, BoardImu, SdCardSpiDevice};

#[allow(unused_imports)]
#[cfg(feature = "airb_omnibus_f4")]
pub use airb_omnibus_f4::{Board, BoardImu, SdCardSpiDevice};

#[cfg(feature = "airb_omnibus_f4_sd")]
pub use airb_omnibus_f4_sd::{Board, BoardImu, SdCardSpiDevice};

#[cfg(feature = "stamp_s3_fly")]
pub use stamp_s3_fly::{Board, BoardImu, SdCardSpiDevice};

#[cfg(feature = "openfc_lite")]
pub use openfc_lite::{Board, BoardImu, SdCardSpiDevice};

#[cfg(feature = "openfc_lite_mini")]
pub use openfc_lite_mini::{Board, BoardImu, SdCardSpiDevice};
