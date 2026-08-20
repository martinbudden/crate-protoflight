pub const UBX_MON_CLASS: u8 = 0x0a;

pub struct UbxMonId;

impl UbxMonId {
    /// Receiver and software version.
    pub const VER: u8 = 0x04;
}
