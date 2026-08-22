/// Ack/Nak Messages: ie Acknowledge or Reject messages to UBX-CFG input messages.
///  Messages in the UBX-ACK class output the processing results to UBX-CFG and some other messages.
pub struct UbxAckId;

impl UbxAckId {
    ///Message not acknowledged.
    pub const NAK: u8 = 0x00;
    /// Message acknowledged.
    pub const ACK: u8 = 0x01;
}
