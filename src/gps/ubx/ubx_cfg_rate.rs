use super::{
    UbxCfgId, UbxClassId,
    ubx_parser::{Parse, UbxParser},
};

/// Poll a message configuration.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxCfgRate {
    pub measurement_rate: u16,
    pub nav_rate: u16,
    pub time_ref: u16,
}

impl Default for UbxCfgRate {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxCfgRate {
    pub const CLASS: UbxClassId = UbxClassId::Cfg;
    pub const ID: u8 = UbxCfgId::RATE;
    pub const PAYLOAD_LEN_U16: u16 = 6;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self { measurement_rate: 0, nav_rate: 0, time_ref: 0 }
    }
}

impl UbxCfgRate {
    pub fn parse(payload: &[u8]) -> Option<UbxCfgRate> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxCfgRate {
            measurement_rate: Parse::try_read_u16(&payload[0..2])?,
            nav_rate: Parse::try_read_u16(&payload[2..4])?,
            time_ref: Parse::try_read_u16(&payload[4..6])?,
        })
    }

    #[inline]
    pub fn make_payload(self) -> [u8; Self::PAYLOAD_LEN] {
        let mut payload = [0u8; Self::PAYLOAD_LEN];

        payload[0..2].copy_from_slice(&self.measurement_rate.to_le_bytes());
        payload[2..4].copy_from_slice(&self.nav_rate.to_le_bytes());
        payload[4..Self::PAYLOAD_LEN].copy_from_slice(&self.time_ref.to_le_bytes());

        payload
    }

    make_frame!();
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxCfgMsgSet {
    pub class: u8,
    pub id: u8,
    pub rate: u8,
}

impl Default for UbxCfgMsgSet {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxCfgMsgSet {
    pub const CLASS: u8 = 0x06;
    pub const ID: u8 = 0x01;
    pub const PAYLOAD_LEN: usize = 2;

    pub const fn new() -> Self {
        Self { class: 0, id: 0, rate: 0 }
    }
}

impl UbxCfgMsgSet {
    pub fn parse(payload: &[u8]) -> Option<UbxCfgMsgSet> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxCfgMsgSet { class: payload[0], id: payload[1], rate: payload[2] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<UbxCfgRate>();
    }
}
