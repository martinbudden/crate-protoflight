use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct SchemaVersion {
    pub version: u8,
}

impl SchemaVersion {
    pub const fn new() -> Self {
        Self { version: 0 }
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct PidProfile {
    pub profile: u8,
}

impl PidProfile {
    pub const _COUNT: usize = 4;

    pub const fn new() -> Self {
        Self { profile: 0 }
    }
}

impl Default for PidProfile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RatesProfile {
    pub profile: u8,
}

impl RatesProfile {
    pub const _COUNT: usize = 4;

    pub const fn new() -> Self {
        Self { profile: 0 }
    }
}

impl Default for RatesProfile {
    fn default() -> Self {
        Self::new()
    }
}
