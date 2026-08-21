pub struct UbxNavId;

impl UbxNavId {
    pub const POS_ECEF: u8 = 0x01;
    pub const POS_LLH: u8 = 0x02;
    pub const STATUS: u8 = 0x03;
    pub const DOP: u8 = 0x04;
    pub const ATT: u8 = 0x05;
    pub const PVT: u8 = 0x07;
    pub const ODO: u8 = 0x09;
    pub const SAT: u8 = 0x10;
    pub const VEL_ECEF: u8 = 0x11;
    pub const VEL_NED: u8 = 0x12;

    pub const SOL: u8 = 0x06; // users are recommended to use the UBX-NAV-PVT message in preference.
    pub const SVINFO: u8 = 0x30; // users are recommended to use the UBX-NAV-SAT message in preference.
}
