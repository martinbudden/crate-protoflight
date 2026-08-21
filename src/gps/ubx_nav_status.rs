use crate::gps::{UbxClassId, ubx_nav::UbxNavId, ubx_parser::Parse};

/// Receiver navigation status.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxNavStatus {
    pub time_of_week_ms: u32,
    pub gps_fix: u8,
    pub flags: u8,
    pub fix_status: u8,
    pub flags2: u8,
    pub time_to_first_fix_ms: u32,
    pub time_since_startup_ms: u32,
}

impl Default for UbxNavStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxNavStatus {
    pub const CLASS: UbxClassId = UbxClassId::Nav;
    pub const ID: u8 = UbxNavId::STATUS;
    pub const PAYLOAD_LEN_U16: u16 = 16;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self {
            time_of_week_ms: 0,
            gps_fix: 0,
            flags: 0,
            fix_status: 0,
            flags2: 0,
            time_to_first_fix_ms: 0,
            time_since_startup_ms: 0,
        }
    }
}

impl UbxNavStatus {
    pub fn parse(payload: &[u8]) -> Option<UbxNavStatus> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxNavStatus {
            time_of_week_ms: Parse::try_read_u32(&payload[0..4])?,
            gps_fix: payload[4],
            flags: payload[5],
            fix_status: payload[6],
            flags2: payload[7],
            time_to_first_fix_ms: Parse::try_read_u32(&payload[8..12])?,
            time_since_startup_ms: Parse::try_read_u32(&payload[12..Self::PAYLOAD_LEN])?,
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
        is_full::<UbxNavStatus>();
    }
}
