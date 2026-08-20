use crate::gps::{
    ubx_nav::{UBX_NAV_CLASS, UbxNavId},
    ubx_parser::Parse,
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
    pub const CLASS: u8 = UBX_NAV_CLASS;
    pub const ID: u8 = UbxNavId::POS_LLH;
    pub const PAYLOAD_LEN: usize = 28;

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
