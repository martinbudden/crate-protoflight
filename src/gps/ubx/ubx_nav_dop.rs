use super::{UbxClassId, UbxNavId, ubx_parser::Parse};

/// Dilution of precision.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxNavDop {
    pub time_of_week_ms: u32,
    pub gdop_x100: u16,
    pub tdop_x100: u16,
    pub pdop_x100: u16,
    pub vdop_x100: u16,
    pub hdop_x100: u16,
    pub ndop_x100: u16,
    pub edop_x100: u16,
}

impl Default for UbxNavDop {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxNavDop {
    pub const CLASS: UbxClassId = UbxClassId::Nav;
    pub const ID: u8 = UbxNavId::DOP;
    pub const PAYLOAD_LEN_U16: u16 = 18;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self {
            time_of_week_ms: 0,
            gdop_x100: 0,
            tdop_x100: 0,
            pdop_x100: 0,
            vdop_x100: 0,
            hdop_x100: 0,
            ndop_x100: 0,
            edop_x100: 0,
        }
    }
}

impl UbxNavDop {
    pub fn parse(payload: &[u8]) -> Option<UbxNavDop> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxNavDop {
            time_of_week_ms: Parse::try_read_u32(&payload[0..4])?,
            gdop_x100: Parse::try_read_u16(&payload[4..6])?,
            tdop_x100: Parse::try_read_u16(&payload[6..8])?,
            pdop_x100: Parse::try_read_u16(&payload[8..10])?,
            vdop_x100: Parse::try_read_u16(&payload[10..12])?,
            hdop_x100: Parse::try_read_u16(&payload[12..14])?,
            ndop_x100: Parse::try_read_u16(&payload[14..16])?,
            edop_x100: Parse::try_read_u16(&payload[16..Self::PAYLOAD_LEN])?,
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
        is_full::<UbxNavDop>();
    }
}
