use crate::gps::{
    nmea::{NmeaGga, NmeaGsa, NmeaGsv, NmeaRmc},
    ubx::{UbxNavDop, UbxNavPvt},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpsStatusData {}

impl Default for GpsStatusData {
    fn default() -> Self {
        Self::new()
    }
}
#[allow(unused)]
impl GpsStatusData {
    pub const fn new() -> Self {
        Self {}
    }
}

impl GpsStatusData {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<GpsStatusData>();
    }
}
