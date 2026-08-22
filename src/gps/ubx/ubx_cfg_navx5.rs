use super::{
    UbxCfgId, UbxClassId,
    ubx_parser::{Parse, UbxParser},
};

/// Navigation engine expert settings.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxCfgNavX5 {
    pub version: u16,
    pub mask1: u16,
    pub mask2: u32,
}

impl Default for UbxCfgNavX5 {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxCfgNavX5 {
    pub const CLASS: UbxClassId = UbxClassId::Cfg;
    pub const ID: u8 = UbxCfgId::RATE;
    pub const PAYLOAD_LEN_U16: u16 = 40;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self { version: 0, mask1: 0, mask2: 0 }
    }
}

impl UbxCfgNavX5 {
    // TODO: UbxCfgNavX5 parse incomplete.
    pub fn parse(payload: &[u8]) -> Option<UbxCfgNavX5> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxCfgNavX5 {
            version: Parse::try_read_u16(&payload[0..2])?,
            mask1: Parse::try_read_u16(&payload[2..4])?,
            mask2: Parse::try_read_u32(&payload[4..8])?,
        })
    }

    #[inline]
    pub fn make_payload(self) -> [u8; Self::PAYLOAD_LEN] {
        let mut payload = [0u8; Self::PAYLOAD_LEN];

        payload[0..2].copy_from_slice(&self.version.to_le_bytes());
        payload[2..4].copy_from_slice(&self.mask1.to_le_bytes());
        payload[4..8].copy_from_slice(&self.mask2.to_le_bytes());

        payload
    }

    make_frame!();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<UbxCfgNavX5>();
    }
}
