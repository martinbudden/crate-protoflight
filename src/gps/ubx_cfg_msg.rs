use crate::gps::{
    ubx_cfg::{UBX_CFG_CLASS, UbxCfgId},
    ubx_parser::Parse,
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
    pub const CLASS: u8 = UBX_CFG_CLASS;
    pub const ID: u8 = UbxCfgId::MSG;
    pub const PAYLOAD_LEN: usize = 2;

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
        is_full::<UbxCfgMsgPoll>();
    }
}
