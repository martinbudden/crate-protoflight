use crate::gps::{
    ubx_nav::{UBX_NAV_CLASS, UbxNavId},
    ubx_parser::Parse,
};

/// Velocity solution in NED frame.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxNavVelNed {
    pub time_of_week_ms: u32,

    pub velocity_north_cmps: i32,
    pub velocity_east_cmps: i32,
    pub velocity_down_cmps: i32,

    pub speed_cmps: u32,
    pub ground_speed_cmps: u32,

    pub heading_degrees_x1e5: i32,

    pub speed_accuracy_cmps: u32,
    pub heading_accuracy_degrees_x1e5: u32,
}

impl Default for UbxNavVelNed {
    fn default() -> Self {
        Self::new()
    }
}

impl UbxNavVelNed {
    pub const CLASS: u8 = UBX_NAV_CLASS;
    pub const ID: u8 = UbxNavId::VEL_NED;
    pub const PAYLOAD_LEN: usize = 36;

    pub const fn new() -> Self {
        Self {
            time_of_week_ms: 0,
            velocity_north_cmps: 0,
            velocity_east_cmps: 0,
            velocity_down_cmps: 0,
            speed_cmps: 0,
            ground_speed_cmps: 0,
            heading_degrees_x1e5: 0,
            speed_accuracy_cmps: 0,
            heading_accuracy_degrees_x1e5: 0,
        }
    }
}

impl UbxNavVelNed {
    pub fn parse(payload: &[u8]) -> Option<UbxNavVelNed> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxNavVelNed {
            time_of_week_ms: Parse::try_read_u32(&payload[0..4])?,

            velocity_north_cmps: Parse::try_read_i32(&payload[4..8])?,
            velocity_east_cmps: Parse::try_read_i32(&payload[8..12])?,
            velocity_down_cmps: Parse::try_read_i32(&payload[12..16])?,
            speed_cmps: Parse::try_read_u32(&payload[16..20])?,
            ground_speed_cmps: Parse::try_read_u32(&payload[20..24])?,

            heading_degrees_x1e5: Parse::try_read_i32(&payload[24..28])?,

            speed_accuracy_cmps: Parse::try_read_u32(&payload[28..32])?,
            heading_accuracy_degrees_x1e5: Parse::try_read_u32(&payload[32..Self::PAYLOAD_LEN])?,
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
        is_full::<UbxNavVelNed>();
    }
}
