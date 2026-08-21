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
