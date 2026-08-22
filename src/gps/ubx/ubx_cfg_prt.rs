use super::{
    UbxCfgId, UbxClassId,
    ubx_parser::{Parse, UbxParser},
};

/// Polls the configuration for one I/O port.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxCfgPrt {
    pub port_id: u8,
}

impl Default for UbxCfgPrt {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxCfgPrt {
    pub const CLASS: UbxClassId = UbxClassId::Cfg;
    pub const ID: u8 = UbxCfgId::PMS;
    pub const PAYLOAD_LEN_U16: u16 = 1;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self { port_id: 0 }
    }
}

impl UbxCfgPrt {
    pub fn parse(payload: &[u8]) -> Option<UbxCfgPrt> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxCfgPrt { port_id: payload[0] })
    }

    #[inline]
    pub fn make_payload(self) -> [u8; Self::PAYLOAD_LEN] {
        let mut payload = [0u8; Self::PAYLOAD_LEN];

        payload[0] = self.port_id;

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
        is_full::<UbxCfgPrt>();
    }
}
