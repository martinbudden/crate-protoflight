use super::{UbxClassId, UbxMonId};

/// Poll receiver and software version.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxMonVer {
    /// Nul-terminated software version string.
    pub software_version: [u8; 30],
    /// Nul-terminated hardware version string.
    pub hardware_version: [u8; 10],
}

impl Default for UbxMonVer {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxMonVer {
    pub const CLASS: UbxClassId = UbxClassId::Mon;
    pub const ID: u8 = UbxMonId::VER;
    pub const PAYLOAD_LEN_U16: u16 = 40; // 40 + 30*N
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self { software_version: [0u8; 30], hardware_version: [0u8; 10] }
    }
}

impl UbxMonVer {
    pub fn parse(payload: &[u8]) -> Option<UbxMonVer> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        // `unwrap` is safe, since we have checked payload.len().
        #[allow(clippy::unwrap_used)]
        Some(UbxMonVer {
            software_version: payload[0..30].try_into().unwrap(),
            hardware_version: payload[30..40].try_into().unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<UbxMonVer>();
    }
}
