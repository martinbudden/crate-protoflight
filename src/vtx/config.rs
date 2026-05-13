use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct VtxConfig {
    pub frequency_mhz: u16,          // sets freq in MHz if band=0
    pub pit_mode_frequency_mhz: u16, // sets out-of-range pit mode frequency
    pub band: u8,                    // 1=A, 2=B, 3=E, 4=F(Airwaves/Fatshark), 5=Raceband
    pub channel: u8,                 // 1-8
    pub power: u8,                   // 0 = lowest
    pub low_power_disarm: u8,        // min power while disarmed, from vtxLowerPowerDisarm_e
    pub softserial_alt: u8,          // prepend 0xff before sending frame even with SOFTSERIAL
}

impl VtxConfig {
    pub const fn new() -> Self {
        Self {
            frequency_mhz: 5740,
            pit_mode_frequency_mhz: 0,
            band: 4,
            channel: 1,
            power: 1,
            low_power_disarm: 1,
            softserial_alt: 0,
        }
    }
}

impl Default for VtxConfig {
    fn default() -> Self {
        Self::new()
    }
}
