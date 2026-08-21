use crate::gps::{
    UbxClassId,
    ubx_cfg::UbxCfgId,
    ubx_parser::{Parse, UbxParser},
};

/// Poll a message configuration.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxCfgPms {
    pub version: u8,
    pub power_setup: u8,
    pub period: u16,
    pub on_time: u16,
    pub reserved1: u16,
}

impl Default for UbxCfgPms {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxCfgPms {
    pub const CLASS: UbxClassId = UbxClassId::Cfg;
    pub const ID: u8 = UbxCfgId::PMS;
    pub const PAYLOAD_LEN_U16: u16 = 8;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self { version: 0, power_setup: 0, period: 0, on_time: 0, reserved1: 0 }
    }
}

impl UbxCfgPms {
    pub fn parse(payload: &[u8]) -> Option<UbxCfgPms> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxCfgPms {
            version: payload[0],
            power_setup: payload[1],
            period: Parse::try_read_u16(&payload[2..4])?,
            on_time: Parse::try_read_u16(&payload[4..6])?,
            reserved1: 0,
        })
    }

    #[inline]
    pub fn make_payload(self) -> [u8; Self::PAYLOAD_LEN] {
        let mut payload = [0u8; Self::PAYLOAD_LEN];

        payload[0] = self.version;
        payload[1] = self.power_setup;

        payload[2..4].copy_from_slice(&self.period.to_le_bytes());
        payload[4..6].copy_from_slice(&self.on_time.to_le_bytes());

        payload
    }

    pub fn make_frame(self) -> [u8; Self::FRAME_LEN] {
        let mut frame = [0u8; Self::FRAME_LEN];

        frame[0] = UbxParser::SYNC_BYTE_1;
        frame[1] = UbxParser::SYNC_BYTE_2;
        frame[2] = Self::CLASS as u8;
        frame[3] = Self::ID;
        let payload_len = Self::PAYLOAD_LEN_U16.to_le_bytes();
        frame[4] = payload_len[0];
        frame[5] = payload_len[1];

        let payload = self.make_payload();
        frame[6..6 + Self::PAYLOAD_LEN].copy_from_slice(&payload);

        // UBX Fletcher checksum covers class, ID, length and payload.
        let mut checksum_a = 0u8;
        let mut checksum_b = 0u8;

        for &byte in &frame[2..Self::FRAME_LEN - 3] {
            checksum_a = checksum_a.wrapping_add(byte);
            checksum_b = checksum_b.wrapping_add(checksum_a);
        }

        frame[Self::FRAME_LEN - 2] = checksum_a;
        frame[Self::FRAME_LEN - 1] = checksum_b;

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
        is_full::<UbxCfgPms>();
    }
}
