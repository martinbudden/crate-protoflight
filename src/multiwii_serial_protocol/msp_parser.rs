use super::crc_dvb_s2::CrcDvbS2;
use super::msp_stream::{MspPacketType, MspVersion};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MspPacket {
    //pending_request: MspPendingSystemRequest,
    //stream_state: MspStreamState,
    pub packet_type: MspPacketType,
    pub msp_version: MspVersion,
    pub cmd: u16,
    pub flags: u8,
    pub data_size: u16,
    pub payload: [u8; Self::BUF_SIZE],
}
/*typedef struct mspPacket_s {
    sbuf_t buf;         // payload only w/o header or crc
    int16_t cmd;
    int16_t result;
    uint8_t flags;      // MSPv2 flags byte. It looks like unused (yet?).
    uint8_t direction;  // It also looks like unused and might be deleted.
} mspPacket_t;*/

impl MspPacket {
    pub const BUF_SIZE: usize = 256;
    pub const fn new() -> Self {
        Self {
            //pending_request: MspPendingSystemRequest::None,
            //stream_state: MspStreamState::Idle,
            packet_type: MspPacketType::Command,
            msp_version: MspVersion::V1,
            cmd: 0,
            flags: 0,
            data_size: 0,
            payload: [0u8; Self::BUF_SIZE],
        }
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload[..self.data_size as usize]
    }
    #[allow(unused)]
    pub fn payload_mut(&mut self) -> &mut [u8] {
        &mut self.payload[..self.data_size as usize]
    }
}

impl Default for MspPacket {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum MspParserState {
    #[default]
    Idle,
    HeaderM,
    HeaderX,
    HeaderV1 {
        offset: usize,
        checksum: u8,
    },
    PayloadV1 {
        len: usize,
        cmd: u8,
        offset: usize,
        checksum: u8,
    },
    HeaderV2 {
        version: MspVersion,
        offset: usize,
        checksum1: u8, // For V2-over-V1
        checksum2: u8, // CRC8
    },
    PayloadV2 {
        version: MspVersion,
        len: usize,
        cmd: u16,
        flags: u8,
        offset: usize,
        checksum1: u8,
        checksum2: u8,
    },
    ChecksumV1 {
        checksum: u8,
    },
    ChecksumV2 {
        version: MspVersion,
        checksum1: u8,
        checksum2: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MspParser {
    parser_state: MspParserState,
    packet: MspPacket,
    pub checksum1: u8,
    pub checksum2: u8,
}

#[allow(unused)]
impl MspParser {
    //const JUMBO_FRAME_SIZE_LIMIT: usize = 255;

    //const MSP_EVALUATE_NON_MSP_DATA: u8 = 0;
    //const MSP_SKIP_NON_MSP_DATA: u8 = 1;
    //const MSP_HEADER_LENGTH: usize = 3;

    //const MSP_MAX_HEADER_SIZE: usize = 9;
}

impl MspParser {
    pub const fn new() -> Self {
        Self {
            //pending_request: MspPendingSystemRequest::None,
            //stream_state: MspStreamState::Idle,
            parser_state: MspParserState::Idle,
            packet: MspPacket::new(),
            checksum1: 0,
            checksum2: 0,
        }
    }
}

impl Default for MspParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MspParser {
    #[allow(clippy::too_many_lines)]
    #[allow(unused)]
    pub fn on_data_received(&mut self, byte: u8) -> Option<MspPacket> {
        let mut complete = false;
        // We take the state out to mutate it, then put it back.
        // This is a common Rust idiom for state machines.
        self.parser_state = match core::mem::take(&mut self.parser_state) {
            MspParserState::Idle => match byte {
                b'M' => MspParserState::HeaderM,
                b'X' => MspParserState::HeaderX,
                _ => MspParserState::Idle,
            },
            MspParserState::HeaderM => match byte {
                b'<' | b'>' => {
                    self.packet.packet_type = if byte == b'<' { MspPacketType::Command } else { MspPacketType::Reply };
                    MspParserState::HeaderV1 { offset: 0, checksum: 0 }
                }
                _ => MspParserState::Idle,
            },
            MspParserState::HeaderX => match byte {
                b'<' | b'>' => {
                    self.packet.packet_type = if byte == b'<' { MspPacketType::Command } else { MspPacketType::Reply };
                    MspParserState::HeaderV2 { version: MspVersion::V2, offset: 0, checksum1: 0, checksum2: 0 }
                }
                _ => MspParserState::Idle,
            },
            MspParserState::HeaderV1 { mut offset, mut checksum } => {
                self.packet.payload[offset] = byte;
                checksum ^= byte;
                offset += 1;
                if offset == 2 {
                    // Size and Cmd byte
                    let size = self.packet.payload[0] as usize;
                    let cmd = self.packet.payload[1];
                    if size > self.packet.payload.len() {
                        MspParserState::Idle
                    } else if cmd == 255 {
                        // V2_FRAME_ID
                        MspParserState::HeaderV2 {
                            version: MspVersion::V2overV1,
                            offset: 2,
                            checksum1: checksum,
                            checksum2: 0,
                        }
                    } else if size > 0 {
                        MspParserState::PayloadV1 { len: size, cmd, offset: 0, checksum }
                    } else {
                        MspParserState::ChecksumV1 { checksum }
                    }
                } else {
                    MspParserState::HeaderV1 { offset, checksum }
                }
            }
            MspParserState::PayloadV1 { len, cmd, mut offset, mut checksum } => {
                self.packet.payload[offset] = byte;
                checksum ^= byte;
                offset += 1;
                if offset == len {
                    self.packet.cmd = u16::from(cmd);
                    MspParserState::ChecksumV1 { checksum }
                } else {
                    MspParserState::PayloadV1 { len, cmd, offset, checksum }
                }
            }
            MspParserState::HeaderV2 { version, mut offset, mut checksum1, mut checksum2 } => {
                self.packet.payload[offset] = byte;
                if version == MspVersion::V2overV1 {
                    checksum1 ^= byte;
                }
                checksum2 = CrcDvbS2::update(checksum2, byte);
                offset += 1;

                // Header size is 5 bytes.
                // V2Native: offset 0..5 (ends at 5)
                // V2overV1: offset 2..7 (ends at 7)
                //let header_len = 5;
                let start_index = if version == MspVersion::V2overV1 { 2 } else { 0 };
                let header_end = start_index + 5;

                if offset == header_end {
                    // Use the start_index to find the V2 fields
                    if start_index + 4 > self.packet.payload.len() {
                        MspParserState::Idle
                    } else {
                        let flags = self.packet.payload[start_index];
                        let cmd = u16::from_le_bytes([
                            self.packet.payload[start_index + 1],
                            self.packet.payload[start_index + 2],
                        ]);
                        let size = u16::from_le_bytes([
                            self.packet.payload[start_index + 3],
                            self.packet.payload[start_index + 4],
                        ]) as usize;
                        if size > self.packet.payload.len() {
                            MspParserState::Idle
                        } else {
                            MspParserState::PayloadV2 {
                                version,
                                len: size,
                                cmd,
                                flags,
                                offset: 0, // Reset to 0 to start filling payload
                                checksum1,
                                checksum2,
                            }
                        }
                    }
                } else {
                    MspParserState::HeaderV2 { version, offset, checksum1, checksum2 }
                }
            }
            MspParserState::PayloadV2 { version, len, cmd, flags, mut offset, mut checksum1, mut checksum2 } => {
                self.packet.payload[offset] = byte;
                if version == MspVersion::V2overV1 {
                    checksum1 ^= byte;
                }
                checksum2 = CrcDvbS2::update(checksum2, byte);
                offset += 1;
                if offset == len {
                    self.packet.cmd = cmd;
                    self.packet.flags = flags;
                    MspParserState::ChecksumV2 { version, checksum1, checksum2 }
                } else {
                    MspParserState::PayloadV2 { version, len, cmd, flags, offset, checksum1, checksum2 }
                }
            }
            MspParserState::ChecksumV1 { checksum } => {
                self.checksum1 = checksum;
                if checksum == byte {
                    complete = true;
                }
                MspParserState::Idle
            }
            MspParserState::ChecksumV2 { version, mut checksum1, checksum2 } => {
                self.checksum2 = checksum2;
                if version == MspVersion::V2overV1 {
                    // V2 over V1 has an extra CRC step
                    if checksum2 == byte {
                        // XOR the CRC byte into the V1 sum before moving to the final check
                        checksum1 ^= byte;
                        MspParserState::ChecksumV1 { checksum: checksum1 }
                    } else {
                        MspParserState::Idle
                    }
                } else {
                    if checksum2 == byte {
                        complete = true;
                    }
                    MspParserState::Idle
                }
            }
        };
        if complete { Some(self.packet) } else { None }
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<MspParserState>();
        is_full::<MspParser>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msp_v1_parsing() {
        let mut parser = MspParser::new();
        let data_packet: [u8; 7] = [
            b'M', b'<', 2,   // Size
            100, // Cmd
            1, 2,   // Payload
            101, // XOR
        ];

        let option = parser.on_data_received(data_packet[0]);
        assert!(option.is_none());
        let option = parser.on_data_received(data_packet[1]);
        assert!(option.is_none());
        let option = parser.on_data_received(data_packet[2]);
        assert!(option.is_none());
        let option = parser.on_data_received(data_packet[3]);
        assert!(option.is_none());
        let option = parser.on_data_received(data_packet[4]);
        assert!(option.is_none());
        let option = parser.on_data_received(data_packet[5]);
        assert!(option.is_none());
        let option = parser.on_data_received(data_packet[6]);

        assert!(option.is_some());
        if let Some(msp_packet) = option {
            assert_eq!(msp_packet.cmd, 100);
            assert_eq!(&parser.packet.payload[0..2], &[1, 2]);
        }

        let mut option = None;
        for &byte in &data_packet {
            option = parser.on_data_received(byte);
            if option.is_some() {
                break;
            }
        }
        assert!(option.is_some());
        if let Some(msp_packet) = option {
            assert_eq!(msp_packet.cmd, 100);
            assert_eq!(&parser.packet.payload[0..2], &[1, 2]);
        }
    }
    #[test]
    fn test_msp_v2_native_parsing() {
        let mut parser = MspParser::new();
        let data_packet: [u8; 10] = [
            b'X', b'<', 0, // Flags
            2, 1, // Cmd (0x0102)
            2, 0, // Size (2)
            0xAA, 0xBB, // Payload
            19,   // CRC8
        ];
        let mut option = None;
        for &byte in &data_packet {
            option = parser.on_data_received(byte);
            if option.is_some() {
                break;
            }
        }
        assert!(option.is_some());
        if let Some(msp_packet) = option {
            assert_eq!(msp_packet.cmd, 0x0102);
            assert_eq!(msp_packet.flags, 0);
            assert_eq!(&parser.packet.payload[0..2], &[0xAA, 0xBB]);
        }

        /*for &byte in &data_packet {
            stream.process_received_packet_data(&mut buf, byte);
        }

        assert_eq!(MspPacketState::CommandReceived, stream.packet_state);
        assert_eq!(19, stream.checksum2);
        assert_eq!(stream.cmd_msp, 0x0102);
        assert_eq!(stream.cmd_flags, 0);
        assert_eq!(&buf[0..2], &[0xAA, 0xBB]);*/
    }
    #[test]
    fn test_msp_v2_over_v1_parsing() {
        let mut parser = MspParser::new();

        // Crafted Packet: V2 Command 0x0102, Flags 0, Payload [0xAA, 0xBB]
        // Encapsulated in V1 (Cmd 255)
        let packet: [u8; 13] = [
            b'M', b'<', // Header
            8,    // V1 Size (5 byte V2 hdr + 2 byte payload + 1 byte V2 CRC)
            255,  // V1 Cmd (V2_FRAME_ID)
            0,    // V2 Flags
            2, 1, // V2 Cmd (0x0102, Little Endian)
            2, 0, // V2 Payload Size (2, Little Endian)
            0xAA, 0xBB, // V2 Payload
            19,   // V2 CRC8 (CRC-8/DVB-S2)
            244,  // V1 XOR Checksum
        ];

        let mut option = None;
        for (i, &byte) in packet.iter().enumerate() {
            option = parser.on_data_received(byte);

            if i < packet.len() - 1 {
                // Ensure we haven't reset to Idle prematurely
                assert!(
                    !matches!(parser.parser_state, MspParserState::Idle),
                    "State machine reset to Idle at byte index {i}",
                );
            }
        }
        assert!(option.is_some());

        if let Some(msp_packet) = option {
            // Final Verification
            assert_eq!(parser.parser_state, MspParserState::Idle);
            assert_eq!(244, parser.checksum1);
            assert_eq!(19, parser.checksum2);
            assert_eq!(msp_packet.cmd, 0x0102);
            assert_eq!(msp_packet.flags, 0);
            assert_eq!(&parser.packet.payload[0..2], &[0xAA, 0xBB]);
        }
    }
}
