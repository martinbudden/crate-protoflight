use super::{UbxAckId, UbxClassId};

/// Message acknowledged.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxAckAck {
    /// Class ID of the Acknowledged Message.
    pub class: u8,
    /// Message ID of the Acknowledged Message.
    pub id: u8,
}

impl Default for UbxAckAck {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxAckAck {
    pub const CLASS: UbxClassId = UbxClassId::Ack;
    pub const ID: u8 = UbxAckId::ACK;
    pub const PAYLOAD_LEN_U16: u16 = 2;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self { class: 0, id: 0 }
    }
}

impl UbxAckAck {
    pub fn parse(payload: &[u8]) -> Option<UbxAckAck> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxAckAck { class: payload[0], id: payload[1] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<UbxAckAck>();
    }
}
