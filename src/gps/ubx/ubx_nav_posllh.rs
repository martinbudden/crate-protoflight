use super::{
    UbxClassId,
    ubx_nav::UbxNavId,
    ubx_parser::{Parse, UbxParser},
};

/// Geodetic position solution.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxNavPosLlh {
    pub time_of_week_ms: u32,
    pub longitude_degrees_x1e7: i32,
    pub latitude_degrees_x1e7: i32,
    pub height_mm: i32,
}

impl Default for UbxNavPosLlh {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxNavPosLlh {
    pub const CLASS: UbxClassId = UbxClassId::Nav;
    pub const ID: u8 = UbxNavId::POS_LLH;
    pub const PAYLOAD_LEN_U16: u16 = 28;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new() -> Self {
        Self { time_of_week_ms: 0, longitude_degrees_x1e7: 0, latitude_degrees_x1e7: 0, height_mm: 0 }
    }
}

impl UbxNavPosLlh {
    pub fn parse(payload: &[u8]) -> Option<UbxNavPosLlh> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxNavPosLlh {
            time_of_week_ms: Parse::try_read_u32(&payload[0..4])?,
            longitude_degrees_x1e7: Parse::try_read_i32(&payload[4..8])?,
            latitude_degrees_x1e7: Parse::try_read_i32(&payload[8..12])?,
            height_mm: Parse::try_read_i32(&payload[12..Self::PAYLOAD_LEN])?,
        })
    }

    #[inline]
    pub fn make_payload(&self) -> [u8; Self::PAYLOAD_LEN] {
        let mut payload = [0u8; Self::PAYLOAD_LEN];

        payload[0..4].copy_from_slice(&self.time_of_week_ms.to_le_bytes());
        payload[4..8].copy_from_slice(&self.longitude_degrees_x1e7.to_le_bytes());
        payload[8..12].copy_from_slice(&self.latitude_degrees_x1e7.to_le_bytes());
        payload[12..Self::PAYLOAD_LEN].copy_from_slice(&self.height_mm.to_le_bytes());

        payload
    }

    pub fn make_frame(&self) -> [u8; Self::FRAME_LEN] {
        let mut frame = [0u8; Self::FRAME_LEN];

        frame[0] = UbxParser::SYNC_BYTE_1;
        frame[1] = UbxParser::SYNC_BYTE_2;
        frame[2] = Self::CLASS as u8;
        frame[3] = Self::ID;
        let payload_len = (Self::PAYLOAD_LEN_U16).to_le_bytes();
        frame[4] = payload_len[0];
        frame[5] = payload_len[1];

        let payload = self.make_payload();
        frame[6..6 + Self::PAYLOAD_LEN].copy_from_slice(&payload);

        // UBX Fletcher checksum covers class, ID, length and payload.
        let mut checksum_a = 0u8;
        let mut checksum_b = 0u8;

        for &byte in &frame[2..Self::FRAME_LEN - 2] {
            checksum_a = checksum_a.wrapping_add(byte);
            checksum_b = checksum_b.wrapping_add(checksum_a);
        }

        frame[Self::FRAME_LEN - 2] = checksum_a;
        frame[Self::FRAME_LEN - 1] = checksum_b;

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
        is_full::<UbxNavPosLlh>();
    }
}
