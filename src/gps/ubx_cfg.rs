pub struct UbxCfgId;

impl UbxCfgId {
    /// Polls the configuration for one I/O port.
    pub const PRT: u8 = 0x00;
    ///  Poll a message configuration/Set message rate.
    pub const MSG: u8 = 0x01;
    /// Navigation engine settings.
    pub const NAV5: u8 = 0x24;
    /// Navigation engine expert settings.
    pub const NAVX5: u8 = 0x03;
    /// Navigation/measurement rate settings.
    pub const RATE: u8 = 0x08;
    /// GNSS system configuration.
    pub const GNSS: u8 = 0x3e;
    /// Power mode setup.
    pub const PMS: u8 = 0x86;
    /// SBAS configuration.
    pub const SBAS: u8 = 0x16;
    /// Extended NMEA protocol configuration V1.
    pub const NMEA: u8 = 0x17;
}
