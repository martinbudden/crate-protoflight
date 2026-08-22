use super::UbxClassId;

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
pub struct UbxMessage<'a> {
    pub class: UbxClassId,
    pub id: u8,
    pub payload: &'a [u8],
}

impl<'a> UbxMessage<'a> {
    pub fn new(class: UbxClassId, id: u8, payload: &'a [u8]) -> Self {
        Self { class, id, payload }
    }
    pub fn new_from_u8_class(class_u8: u8, id: u8, payload: &'a [u8]) -> Option<Self> {
        UbxClassId::try_from_u8(class_u8).map(|class| Self { class, id, payload })
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum UbxState {
    #[default]
    WaitingForSync1,
    WaitingForSync2,
    Class,
    Id,
    LengthLow,
    LengthHigh,
    Payload,
    ChecksumA,
    ChecksumB,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UbxParser {
    state: UbxState,

    class: u8,
    id: u8,
    payload_length: usize,
    payload_index: usize,

    payload: [u8; Self::MAX_PAYLOAD_SIZE],

    checksum_a: u8,
    checksum_b: u8,
    received_checksum_a: u8,
}

impl Default for UbxParser {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl UbxParser {
    pub const MAX_PAYLOAD_SIZE: usize = 256;
    pub const SYNC_BYTE_1: u8 = 0xB5;
    pub const SYNC_BYTE_2: u8 = 0x62;

    pub const fn new() -> Self {
        Self {
            state: UbxState::WaitingForSync1,
            class: 0,
            id: 0,
            payload_length: 0,
            payload_index: 0,
            payload: [0; Self::MAX_PAYLOAD_SIZE],
            checksum_a: 0,
            checksum_b: 0,
            received_checksum_a: 0,
        }
    }

    fn reset(&mut self) {
        self.state = UbxState::WaitingForSync1;
        self.payload_length = 0;
        self.payload_index = 0;
        self.checksum_a = 0;
        self.checksum_b = 0;
        self.received_checksum_a = 0;
    }

    #[inline]
    fn update_checksum(&mut self, byte: u8) {
        self.checksum_a = self.checksum_a.wrapping_add(byte);
        self.checksum_b = self.checksum_b.wrapping_add(self.checksum_a);
    }

    pub fn on_data_received(&mut self, byte: u8) -> Option<UbxMessage<'_>> {
        let mut complete = false;

        self.state = match core::mem::take(&mut self.state) {
            UbxState::WaitingForSync1 => {
                if byte == Self::SYNC_BYTE_1 {
                    UbxState::WaitingForSync2
                } else {
                    UbxState::WaitingForSync1
                }
            }

            UbxState::WaitingForSync2 => {
                if byte == Self::SYNC_BYTE_2 {
                    UbxState::Class
                } else if byte == Self::SYNC_BYTE_1 {
                    // Stay here so a repeated sync byte can begin a frame.
                    UbxState::WaitingForSync2
                } else {
                    UbxState::WaitingForSync1
                }
            }

            UbxState::Class => {
                self.class = byte;
                self.checksum_a = byte;
                self.checksum_b = self.checksum_a;
                UbxState::Id
            }

            UbxState::Id => {
                self.id = byte;
                self.update_checksum(byte);
                UbxState::LengthLow
            }

            UbxState::LengthLow => {
                self.payload_length = usize::from(byte);
                self.update_checksum(byte);
                UbxState::LengthHigh
            }

            UbxState::LengthHigh => {
                self.payload_length |= usize::from(byte) << 8;
                self.update_checksum(byte);

                if self.payload_length > Self::MAX_PAYLOAD_SIZE {
                    UbxState::WaitingForSync1
                } else if self.payload_length == 0 {
                    UbxState::ChecksumA
                } else {
                    self.payload_index = 0;
                    UbxState::Payload
                }
            }

            UbxState::Payload => {
                self.payload[self.payload_index] = byte;
                self.payload_index += 1;
                self.update_checksum(byte);

                if self.payload_index == self.payload_length { UbxState::ChecksumA } else { UbxState::Payload }
            }

            UbxState::ChecksumA => {
                self.received_checksum_a = byte;
                UbxState::ChecksumB
            }

            UbxState::ChecksumB => {
                if self.received_checksum_a == self.checksum_a && byte == self.checksum_b {
                    complete = true;
                    UbxState::WaitingForSync1
                } else {
                    UbxState::WaitingForSync1
                }
            }
        };

        if complete {
            UbxMessage::new_from_u8_class(self.class, self.id, &self.payload[..self.payload_length])
        } else {
            None
        }
    }
}

pub struct Parse;

impl Parse {
    #[inline]
    pub fn try_read_u32(bytes: &[u8]) -> Option<u32> {
        let bytes: [u8; 4] = bytes.get(..4)?.try_into().ok()?;
        Some(u32::from_le_bytes(bytes))
    }

    #[inline]
    pub fn try_read_i32(bytes: &[u8]) -> Option<i32> {
        let bytes: [u8; 4] = bytes.get(..4)?.try_into().ok()?;
        Some(i32::from_le_bytes(bytes))
    }

    #[inline]
    pub fn try_read_u16(bytes: &[u8]) -> Option<u16> {
        let bytes: [u8; 2] = bytes.get(..2)?.try_into().ok()?;
        Some(u16::from_le_bytes(bytes))
    }

    #[inline]
    pub fn try_read_i16(bytes: &[u8]) -> Option<i16> {
        let bytes: [u8; 2] = bytes.get(..2)?.try_into().ok()?;
        Some(i16::from_le_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn valid_empty_message_is_received() {
        let frame = [
            0xB5, 0x62, // sync
            0x01, 0x07, // NAV-PVT
            0x00, 0x00, // zero-length payload
            0x08, 0x19, // checksum
        ];
        let mut parser = UbxParser::new();
        let mut result = None;
        for byte in frame {
            result = parser.on_data_received(byte);
        }
        assert_eq!(result, Some(UbxMessage { class: UbxClassId::Nav, id: 0x07, payload: &[] }));
    }
    #[test]
    fn valid_message_with_payload_is_received() {
        let frame = [0xB5, 0x62, 0x01, 0x02, 0x03, 0x00, 0x11, 0x22, 0x33, 0x6C, 0xCC];
        let mut parser = UbxParser::new();
        let mut result = None;
        for byte in frame {
            assert!(result.is_none());
            result = parser.on_data_received(byte);
        }
        assert_eq!(result, Some(UbxMessage { class: UbxClassId::Nav, id: 0x02, payload: &[0x11, 0x22, 0x33] }));
    }
    #[test]
    fn invalid_checksum_is_rejected() {
        let frame = [
            0xB5, 0x62, 0x01, 0x02, 0x03, 0x00, 0x11, 0x22, 0x33, 0x6C, 0xCD, // CK_B deliberately wrong
        ];
        let mut parser = UbxParser::new();
        let mut result = None;
        for byte in frame {
            result = parser.on_data_received(byte);
        }
        assert_eq!(result, None);
    }
    #[test]
    fn parser_recovers_after_invalid_checksum() {
        let bad_frame = [
            0xB5, 0x62, 0x01, 0x02, 0x03, 0x00, 0x11, 0x22, 0x33, 0x6C, 0xCD, // Bad CK_B
        ];
        let good_frame = [
            0xB5, 0x62, 0x01, 0x02, 0x03, 0x00, 0x11, 0x22, 0x33, 0x6C, 0xCC, // Correct checksum
        ];
        let mut parser = UbxParser::new();
        for byte in bad_frame {
            assert!(parser.on_data_received(byte).is_none());
        }
        let mut result = None;
        for byte in good_frame {
            result = parser.on_data_received(byte);
        }
        assert_eq!(result, Some(UbxMessage { class: UbxClassId::Nav, id: 0x02, payload: &[0x11, 0x22, 0x33] }));
    }
    #[test]
    fn oversized_payload_is_rejected() {
        let payload_length = UbxParser::MAX_PAYLOAD_SIZE + 1;

        #[allow(clippy::cast_possible_truncation)]
        let length_bytes = payload_length.to_le_bytes();

        let frame = [
            0xB5,
            0x62, // sync
            0x01,
            0x02, // class, ID
            length_bytes[0],
            length_bytes[1],
        ];
        let mut parser = UbxParser::new();
        for byte in frame {
            assert!(parser.on_data_received(byte).is_none());
        }
    }
    #[test]
    fn oversized_payload_is_rejected_and_parser_recovers() {
        #[allow(clippy::cast_possible_truncation)]
        let length_bytes = (UbxParser::MAX_PAYLOAD_SIZE as u16 + 1).to_le_bytes();
        let oversized_frame_start = [0xB5, 0x62, 0x01, 0x02, length_bytes[0], length_bytes[1]];
        let good_frame = [0xB5, 0x62, 0x01, 0x02, 0x03, 0x00, 0x11, 0x22, 0x33, 0x6C, 0xCC];
        let mut parser = UbxParser::new();
        for byte in oversized_frame_start {
            assert!(parser.on_data_received(byte).is_none());
        }
        let mut result = None;
        for byte in good_frame {
            result = parser.on_data_received(byte);
        }
        assert_eq!(result, Some(UbxMessage { class: UbxClassId::Nav, id: 0x02, payload: &[0x11, 0x22, 0x33] }));
    }
}
