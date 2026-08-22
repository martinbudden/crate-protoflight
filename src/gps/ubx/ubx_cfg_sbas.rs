use super::{
    UbxCfgId, UbxClassId,
    ubx_parser::{Parse, UbxParser},
};

/// SBAS configuration.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxCfgSbas {
    pub mode: u8,
    pub usage: u8,
    pub max_sbas: u8,
    pub scanmode2: u8,
    pub scanmode1: u32,
}

impl Default for UbxCfgSbas {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxCfgSbas {
    pub const CLASS: UbxClassId = UbxClassId::Cfg;
    pub const ID: u8 = UbxCfgId::PMS;
    pub const PAYLOAD_LEN_U16: u16 = 8;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self { mode: 0, usage: 0, max_sbas: 0, scanmode2: 0, scanmode1: 0 }
    }
}

impl UbxCfgSbas {
    pub fn parse(payload: &[u8]) -> Option<UbxCfgSbas> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxCfgSbas {
            mode: payload[0],
            usage: payload[1],
            max_sbas: payload[2],
            scanmode2: payload[3],
            scanmode1: Parse::try_read_u32(&payload[4..8])?,
        })
    }

    #[inline]
    pub fn make_payload(self) -> [u8; Self::PAYLOAD_LEN] {
        let mut payload = [0u8; Self::PAYLOAD_LEN];

        payload[0] = self.mode;
        payload[1] = self.usage;
        payload[2] = self.max_sbas;
        payload[3] = self.scanmode2;

        payload[4..8].copy_from_slice(&self.scanmode1.to_le_bytes());

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
        is_full::<UbxCfgSbas>();
    }
}
