#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum UbxVersion {
    M5,
    M6,
    M7,
    M8,
    M9,
    M10,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum UbxClassId {
    /// Navigation Results Messages: Position, Speed, Time, Acceleration, Heading, DOP, SVs used.
    Nav = 0x01,
    /// Receiver Manager Messages: Satellite Status, RTC Status.
    Rxm = 0x02,
    /// Information Messages: Printf-Style Messages, with IDs such as Error, Warning, Notice.
    Inf = 0x04,
    ///Ack/Nak Messages: Acknowledge or Reject messages to UBX-CFG input messages.
    Ack = 0x05,
    /// Configuration Input Messages: Configure the receiver.
    Cfg = 0x06,
    /// Firmware Update Messages: Memory/Flash erase/write, Reboot, Flash identification, etc.
    Upd = 0x09,
    /// Monitoring Messages: Communication Status, Stack Usage, Task Status.
    Mon = 0x0A,
    /// Assist Now Aiding Messages: Ephemeris, Almanac, other A-GPS data input.
    Aid = 0x0B,
    /// Timing Messages: Time Pulse Output, Time Mark Results.
    Tim = 0x0D,
    /// External Sensor Fusion Messages: External Sensor Measurements and Status Information.
    Esf = 0x10,
    /// Multiple GNSS Assistance Messages: Assistance data for various GNSS.
    Mga = 0x13,
    /// Logging Messages: Log creation, deletion, info and retrieval.
    Log = 0x21,
    /// Security Feature Messages.
    Sec = 0x27,
    /// High Rate Navigation Results Messages: High rate time, position, speed, heading.
    Hnr = 0x28,
    // For configuring NMEA messages using the UBX protocol message UBX-CFG-MSG.
    Nmea = 0xf0,
}

impl UbxClassId {
    pub fn try_from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Nav),
            0x02 => Some(Self::Rxm),
            0x04 => Some(Self::Inf),
            0x05 => Some(Self::Ack),
            0x06 => Some(Self::Cfg),
            0x09 => Some(Self::Upd),
            0x0a => Some(Self::Mon),
            0x0b => Some(Self::Aid),
            0x0d => Some(Self::Tim),
            0x10 => Some(Self::Esf),
            0x13 => Some(Self::Mga),
            0x21 => Some(Self::Log),
            0x27 => Some(Self::Sec),
            0x28 => Some(Self::Hnr),
            _ => None,
        }
    }
}

/// Ack/Nak Messages: ie Acknowledge or Reject messages to UBX-CFG input messages.
///  Messages in the UBX-ACK class output the processing results to UBX-CFG and some other messages.
pub struct UbxAckId;

impl UbxAckId {
    ///Message not acknowledged.
    pub const NAK: u8 = 0x00;
    /// Message acknowledged.
    pub const ACK: u8 = 0x01;
}
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

pub struct UbxMonId;

impl UbxMonId {
    /// Receiver and software version.
    pub const VER: u8 = 0x04;
}

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

pub struct UbxNmeaId;

impl UbxNmeaId {
    pub const _DTM: u8 = 0x0A; // Datum reference
    pub const _GBQ: u8 = 0x44; // Poll a standard message (Talker ID GB)
    pub const _GBS: u8 = 0x09; // GNSS satellite fault detection
    pub const GGA: u8 = 0x00; // Global positioning system fix data
    pub const GLL: u8 = 0x01; // Latitude and longitude, with time of position fix and status
    pub const _GLQ: u8 = 0x43; // Poll a standard message (Talker ID GL)
    pub const _GNQ: u8 = 0x42; // Poll a standard message (Talker ID GN)
    pub const _GNS: u8 = 0x0D; // GNSS fix data
    pub const _GPQ: u8 = 0x40; // Poll a standard message (Talker ID GP)
    pub const _GRS: u8 = 0x06; // GNSS range residuals
    pub const GSA: u8 = 0x02; // GNSS DOP and active satellites
    pub const _GST: u8 = 0x07; // GNSS pseudo-range error statistics
    pub const GSV: u8 = 0x03; // GNSS satellites in view
    pub const RMC: u8 = 0x04; // Recommended minimum data
    pub const _THS: u8 = 0x0E; // True heading and status
    pub const _TXT: u8 = 0x41; // Text transmission
    pub const _VLW: u8 = 0x0F; // Dual ground/water distance
    pub const VTG: u8 = 0x05; // Course over ground and ground speed
    pub const _ZDA: u8 = 0x08; // Time and date
}
