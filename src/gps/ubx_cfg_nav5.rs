use crate::gps::{
    UbxClassId,
    ubx_cfg::UbxCfgId,
    ubx_parser::{Parse, UbxParser},
};

/// Poll a message configuration.
#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxCfgNav5 {
    pub mask: u16,
    pub dynamic_platform_model: u8,
    pub fixing_mode: u8,
    pub fixed_altitude_msl_m_x100: i32,
    pub fixed_altitude_variance_m2_x100000: i32,
    pub min_elevation_degrees: i8,
    pub dr_limit: u8, // reserved
    pub pdop_x10: u16,
    pub tdop_x10: u16,
    pub p_accuracy_mask: u16,
    pub t_accuracy_mask: u16,
    pub static_hold_threshold_cmps: u8,
    pub dgnss_timeout_s: u8,
    pub cno_threshold_sv_count: u8,
    pub cno_threshold_dbhz: u8,
    pub reserved1: u16,
    pub static_hold_distance_threshold_m: u16,
    pub utc_standard: u8,
    pub reserved2: [u8; 5],
}

impl Default for UbxCfgNav5 {
    fn default() -> Self {
        Self::new(0)
    }
}

impl UbxCfgNav5 {
    pub const CLASS: UbxClassId = UbxClassId::Cfg;
    pub const ID: u8 = UbxCfgId::RATE;
    pub const PAYLOAD_LEN_U16: u16 = 36;
    pub const PAYLOAD_LEN: usize = Self::PAYLOAD_LEN_U16 as usize;
    pub const FRAME_LEN: usize = Self::PAYLOAD_LEN + 8;

    pub const fn new(model: u8) -> Self {
        Self {
            mask: 0,
            dynamic_platform_model: 0,
            fixing_mode: 0,
            fixed_altitude_msl_m_x100: 0,
            fixed_altitude_variance_m2_x100000: 0,
            min_elevation_degrees: 0,
            dr_limit: 0, // reserved
            pdop_x10: 0,
            tdop_x10: 0,
            p_accuracy_mask: 0,
            t_accuracy_mask: 0,
            static_hold_threshold_cmps: 0,
            dgnss_timeout_s: 0,
            cno_threshold_sv_count: 0,
            cno_threshold_dbhz: 0,
            reserved1: 0,
            static_hold_distance_threshold_m: 0,
            utc_standard: 0,
            reserved2: [0u8; 5],
        }
    }
}

impl UbxCfgNav5 {
    pub fn parse(payload: &[u8]) -> Option<UbxCfgNav5> {
        if payload.len() != Self::PAYLOAD_LEN {
            return None;
        }
        Some(UbxCfgNav5 {
            mask: Parse::try_read_u16(&payload[0..2])?,
            dynamic_platform_model: payload[2],
            fixing_mode: payload[3],
            fixed_altitude_msl_m_x100: Parse::try_read_i32(&payload[4..8])?,
            fixed_altitude_variance_m2_x100000: Parse::try_read_i32(&payload[8..12])?,
            min_elevation_degrees: payload[12].cast_signed(),
            dr_limit: payload[13], // reserved
            pdop_x10: Parse::try_read_u16(&payload[14..16])?,
            tdop_x10: Parse::try_read_u16(&payload[16..18])?,
            p_accuracy_mask: Parse::try_read_u16(&payload[18..20])?,
            t_accuracy_mask: Parse::try_read_u16(&payload[20..22])?,
            static_hold_threshold_cmps: payload[22],
            dgnss_timeout_s: payload[23],
            cno_threshold_sv_count: payload[24],
            cno_threshold_dbhz: payload[25],
            reserved1: Parse::try_read_u16(&payload[26..28])?,
            static_hold_distance_threshold_m: Parse::try_read_u16(&payload[28..32])?,
            utc_standard: payload[30],
            reserved2: [0u8; 5],
        })
    }

    #[inline]
    pub fn make_payload(self) -> [u8; Self::PAYLOAD_LEN] {
        let mut payload = [0u8; Self::PAYLOAD_LEN];

        payload[0..2].copy_from_slice(&self.mask.to_le_bytes());
        payload[2] = self.dynamic_platform_model;
        payload[3] = self.fixing_mode;
        payload[4..8].copy_from_slice(&self.fixed_altitude_msl_m_x100.to_le_bytes());
        payload[8..12].copy_from_slice(&self.fixed_altitude_variance_m2_x100000.to_le_bytes());
        payload[12] = self.min_elevation_degrees.cast_unsigned();
        payload[13] = self.dr_limit; // reserved
        payload[14..16].copy_from_slice(&self.pdop_x10.to_le_bytes());
        payload[16..18].copy_from_slice(&self.tdop_x10.to_le_bytes());
        payload[18..20].copy_from_slice(&self.p_accuracy_mask.to_le_bytes());
        payload[20..22].copy_from_slice(&self.t_accuracy_mask.to_le_bytes());
        payload[22] = self.static_hold_threshold_cmps;
        payload[23] = self.dgnss_timeout_s;
        payload[24] = self.cno_threshold_sv_count;
        payload[25] = self.cno_threshold_dbhz;
        payload[26..28].copy_from_slice(&self.reserved1.to_le_bytes());
        payload[28..30].copy_from_slice(&self.static_hold_distance_threshold_m.to_le_bytes());
        payload[30] = self.utc_standard;

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
        is_full::<UbxCfgNav5>();
    }
}
