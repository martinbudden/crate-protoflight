use crate::gps::{
    nmea::{NmeaGga, NmeaGsa, NmeaGsv, NmeaRmc},
    ubx::{UbxNavDop, UbxNavPvt, UbxVersion},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpsStatus {
    pub errors: u32,
    pub timeouts: u32,
    pub last_nav_message_time_of_week: u32,
    pub last_message_sent: u32,
    pub ack_waiting_message_id: u8,
    pub ack_state: UbxAckState,
    pub update_rate_hz: u8,
    pub ubx_version: UbxVersion,
}

impl Default for GpsStatus {
    fn default() -> Self {
        Self::new()
    }
}
#[allow(unused)]
impl GpsStatus {
    pub const fn new() -> Self {
        Self {
            errors: 0,
            timeouts: 0,
            last_nav_message_time_of_week: 0,
            last_message_sent: 0,
            ack_waiting_message_id: 0,
            ack_state: UbxAckState::Idle,
            update_rate_hz: 10,
            ubx_version: UbxVersion::M8,
        }
    }
}

impl GpsStatus {
    pub fn amend_with_nmea_gga(&mut self, gga: NmeaGga) {
        _ = self;
        _ = gga;
    }

    // TODO: amend_with_gsa
    pub fn amend_with_nmea_gsa(&mut self, gsa: NmeaGsa) {
        _ = self;
        _ = gsa;
    }

    // TODO: amend_with_gsa
    #[allow(unused)]
    pub fn amend_with_nmea_gsv(&mut self, gsv: NmeaGsv) {
        _ = self;
        _ = gsv;
    }

    // TODO: amend_with_rmc
    pub fn amend_with_nmea_rmc(&mut self, gsv: NmeaRmc) {
        _ = self;
        _ = gsv;
    }

    pub fn amend_with_ubx_nav_pvt(&mut self, nav: UbxNavPvt) {
        _ = self;
        _ = nav;
    }
    pub fn amend_with_ubx_nav_dop(&mut self, dop: UbxNavDop) {
        _ = self;
        _ = dop;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum UbxAckState {
    #[default]
    Idle = 0,
    Waiting = 1,
    GotAck = 2,
    GotNack = 3,
}

#[allow(unused)]
impl UbxAckState {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Idle,
            1 => Self::Waiting,
            2 => Self::GotAck,
            3 => Self::GotNack,
            _ => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<GpsStatus>();
    }
}
