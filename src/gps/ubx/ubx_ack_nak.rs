use super::{UbxAckId, UbxClassId};

/// Message acknowledged.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxAckNak {
    /// Class ID of the Acknowledged Message.
    pub class: u8,
    /// Message ID of the Acknowledged Message.
    pub id: u8,
}

impl Default for UbxAckNak {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxAckNak {
    pub const CLASS: UbxClassId = UbxClassId::Ack;
    pub const ID: u8 = UbxAckId::NAK;
    pub const PAYLOAD_LEN_U16: u16 = 2;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self { class: 0, id: 0 }
    }
}

impl UbxAckNak {
    pub fn parse(payload: &[u8]) -> Option<UbxAckNak> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxAckNak { class: payload[0], id: payload[1] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<UbxAckNak>();
    }
}
