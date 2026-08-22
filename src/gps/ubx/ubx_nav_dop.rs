use super::{UbxClassId, UbxNavId, ubx_parser::Parse};

/// Dilution of precision.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxNavDop {
    pub time_of_week_ms: u32,
    pub g_dop: u16,
    pub t_dop: u16,
    pub p_dop: u16,
    pub v_dop: u16,
    pub h_dop: u16,
    pub n_dop: u16,
    pub e_dop: u16,
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
        Self { time_of_week_ms: 0, g_dop: 0, t_dop: 0, p_dop: 0, v_dop: 0, h_dop: 0, n_dop: 0, e_dop: 0 }
    }
}

impl UbxNavDop {
    pub fn parse(payload: &[u8]) -> Option<UbxNavDop> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxNavDop {
            time_of_week_ms: Parse::try_read_u32(&payload[0..4])?,
            g_dop: Parse::try_read_u16(&payload[4..6])?,
            t_dop: Parse::try_read_u16(&payload[6..8])?,
            p_dop: Parse::try_read_u16(&payload[8..10])?,
            v_dop: Parse::try_read_u16(&payload[10..12])?,
            h_dop: Parse::try_read_u16(&payload[12..14])?,
            n_dop: Parse::try_read_u16(&payload[14..16])?,
            e_dop: Parse::try_read_u16(&payload[16..Self::PAYLOAD_LEN])?,
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
