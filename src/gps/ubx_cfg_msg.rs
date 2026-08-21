use crate::gps::{
    UbxClassId,
    ubx_cfg::UbxCfgId,
    ubx_parser::{Parse, UbxParser},
};

/// Poll a message configuration.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxCfgMsgPoll {
    pub class: u8,
    pub id: u8,
}

impl Default for UbxCfgMsgPoll {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxCfgMsgPoll {
    pub const CLASS: UbxClassId = UbxClassId::Cfg;
    pub const ID: u8 = UbxCfgId::MSG;
    pub const PAYLOAD_LEN_U16: u16 = 2;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self { class: 0, id: 0 }
    }
}

impl UbxCfgMsgPoll {
    pub fn parse(payload: &[u8]) -> Option<UbxCfgMsgPoll> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxCfgMsgPoll { class: payload[0], id: payload[1] })
    }

    #[inline]
    pub fn make_payload(self) -> [u8; Self::PAYLOAD_LEN] {
        let mut payload = [0u8; Self::PAYLOAD_LEN];

        payload[0] = self.class;
        payload[1] = self.id;

        payload
    }

    pub fn make_frame(self) -> [u8; Self::FRAME_LEN] {
        let mut frame = [0u8; Self::FRAME_LEN];

        frame[0..4].copy_from_slice(&[UbxParser::SYNC_BYTE_1, UbxParser::SYNC_BYTE_2, Self::CLASS as u8, Self::ID]);
        frame[4..6].copy_from_slice(&Self::PAYLOAD_LEN_U16.to_le_bytes());

        frame[6..6 + Self::PAYLOAD_LEN].copy_from_slice(&self.make_payload());

        // UBX Fletcher checksum covers class, ID, length and payload.
        let mut checksum = [0u8; 2];
        for &byte in &frame[2..Self::FRAME_LEN - 3] {
            checksum[0] = checksum[0].wrapping_add(byte);
            checksum[1] = checksum[1].wrapping_add(checksum[1]);
        }
        frame[Self::FRAME_LEN - 2..Self::FRAME_LEN].copy_from_slice(&checksum);

        frame
    }
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
    pub const CLASS: UbxClassId = UbxClassId::Cfg;
    pub const ID: u8 = UbxCfgId::MSG;
    pub const PAYLOAD_LEN_U16: u16 = 3;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

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
    #[inline]
    pub fn make_payload(self) -> [u8; Self::PAYLOAD_LEN] {
        let mut payload = [0u8; Self::PAYLOAD_LEN];

        payload[0] = self.class;
        payload[1] = self.id;
        payload[2] = self.rate;

        payload
    }

    pub fn make_frame(self) -> [u8; Self::FRAME_LEN] {
        let mut frame = [0u8; Self::FRAME_LEN];

        frame[0..4].copy_from_slice(&[UbxParser::SYNC_BYTE_1, UbxParser::SYNC_BYTE_2, Self::CLASS as u8, Self::ID]);
        frame[4..6].copy_from_slice(&Self::PAYLOAD_LEN_U16.to_le_bytes());

        frame[6..6 + Self::PAYLOAD_LEN].copy_from_slice(&self.make_payload());

        // UBX Fletcher checksum covers class, ID, length and payload.
        let mut checksum = [0u8; 2];
        for &byte in &frame[2..Self::FRAME_LEN - 3] {
            checksum[0] = checksum[0].wrapping_add(byte);
            checksum[1] = checksum[1].wrapping_add(checksum[1]);
        }
        frame[Self::FRAME_LEN - 2..Self::FRAME_LEN].copy_from_slice(&checksum);

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<UbxCfgMsgPoll>();
    }
}
