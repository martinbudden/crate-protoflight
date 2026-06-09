#[derive(Clone, Copy, Default, Debug, PartialEq)]
#[allow(unused)]
pub enum MspVersion {
    #[default]
    V1,
    V2overV1,
    V2,
}
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum MspPacketType {
    #[default]
    Command,
    Reply,
}

/*
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum MspStreamState {
    #[default]
    Idle,
    MspPacket,
    CliActive,
    CliCmd,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum MspPendingSystemRequest {
    #[default]
    None,
    BootloaderRom,
    Cli,
    BootloaderFlash,
}



#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct MspStreamHeaderV2 {
    pub flags: u8,
    pub cmd: u16,
    pub size: u16,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct MspStreamPacketWithHeader {
    pub hdr_buf: [u8; 16],
    pub crc_buf: [u8; 2],
    //const uint8_t* data_ptr;
    pub data_len: u16,
    pub crc_len: u16,
    pub hrd_len: u16,
    pub checksum: u8,
}

impl MspStreamPacketWithHeader {
    const HDR_LEN: u8 = 3;
}
*/

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MspStream {
    //pending_request: MspPendingSystemRequest,
    //stream_state: MspStreamState,
    packet_state: MspPacketState,
    packet_type: MspPacketType,
    msp_version: MspVersion,
    pub cmd_msp: u16,
    pub offset: u16,
    pub data_size: u16,
    pub cmd_flags: u8,
    pub checksum1: u8,
    pub checksum2: u8,
    pub in_buf: [u8; Self::INBUF_SIZE],
    pub out_buf: [u8; Self::OUTBUF_SIZE],
}

#[allow(unused)]
impl MspStream {
    const JUMBO_FRAME_SIZE_LIMIT: usize = 255;

    const MSP_EVALUATE_NON_MSP_DATA: u8 = 0;
    const MSP_SKIP_NON_MSP_DATA: u8 = 1;
    const MSP_HEADER_LENGTH: usize = 3;
    const INBUF_SIZE: usize = 192;
    const OUTBUF_SIZE_MIN: usize = 512; // As of 2021/08/10 MSP_BOX_NAMES generates a 307 byte response for page 1. There has been overflow issues with 320 byte buffer.
    const DATAFLASH_BUFFER_SIZE: usize = 4096;
    const DATAFLASH_INFO_SIZE: usize = 16;
    //const Self::OUTBUF_SIZE:usize = Self::DATAFLASH_BUFFER_SIZE + Self::DATAFLASH_INFO_SIZE;
    const OUTBUF_SIZE: usize = Self::OUTBUF_SIZE_MIN;

    const MSP_MAX_HEADER_SIZE: usize = 9;
}

impl MspStream {
    pub const fn new() -> Self {
        Self {
            //pending_request: MspPendingSystemRequest::None,
            //stream_state: MspStreamState::Idle,
            packet_state: MspPacketState::Idle,
            packet_type: MspPacketType::Command,
            msp_version: MspVersion::V1,
            cmd_msp: 0,
            offset: 0,
            data_size: 0,
            cmd_flags: 0,
            checksum1: 0,
            checksum2: 0,
            in_buf: [0u8; MspStream::INBUF_SIZE],
            out_buf: [0u8; MspStream::OUTBUF_SIZE],
        }
    }
}

impl Default for MspStream {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard CRC-8/DVB-S2 update function.
pub fn crc8_dvb_s2(mut crc: u8, byte: u8) -> u8 {
    crc ^= byte;
    for _ in 0..8 {
        if crc & 0x80 != 0 {
            crc = (crc << 1) ^ 0xD5;
        } else {
            crc <<= 1;
        }
    }
    crc
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
#[allow(unused)]
pub enum MspPacketState {
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
    CommandReceived,
}

#[allow(unused)]
impl MspStream {
    #[allow(clippy::too_many_lines)]
    pub fn process_received_packet_data(&mut self, c: u8) {
        // We take the state out to mutate it, then put it back.
        // This is a common Rust idiom for state machines.
        self.packet_state = match core::mem::take(&mut self.packet_state) {
            MspPacketState::Idle | MspPacketState::CommandReceived => match c {
                b'M' => MspPacketState::HeaderM,
                b'X' => MspPacketState::HeaderX,
                _ => MspPacketState::Idle,
            },
            MspPacketState::HeaderM => match c {
                b'<' | b'>' => {
                    self.packet_type = if c == b'<' { MspPacketType::Command } else { MspPacketType::Reply };
                    MspPacketState::HeaderV1 { offset: 0, checksum: 0 }
                }
                _ => MspPacketState::Idle,
            },
            MspPacketState::HeaderX => match c {
                b'<' | b'>' => {
                    self.packet_type = if c == b'<' { MspPacketType::Command } else { MspPacketType::Reply };
                    MspPacketState::HeaderV2 { version: MspVersion::V2, offset: 0, checksum1: 0, checksum2: 0 }
                }
                _ => MspPacketState::Idle,
            },
            MspPacketState::HeaderV1 { mut offset, mut checksum } => {
                self.in_buf[offset] = c;
                checksum ^= c;
                offset += 1;
                if offset == 2 {
                    // Size and Cmd byte
                    let size = self.in_buf[0] as usize;
                    let cmd = self.in_buf[1];
                    if size > MspStream::INBUF_SIZE {
                        MspPacketState::Idle
                    } else if cmd == 255 {
                        // V2_FRAME_ID
                        MspPacketState::HeaderV2 {
                            version: MspVersion::V2overV1,
                            offset: 2,
                            checksum1: checksum,
                            checksum2: 0,
                        }
                    } else if size > 0 {
                        MspPacketState::PayloadV1 { len: size, cmd, offset: 0, checksum }
                    } else {
                        MspPacketState::ChecksumV1 { checksum }
                    }
                } else {
                    MspPacketState::HeaderV1 { offset, checksum }
                }
            }
            MspPacketState::PayloadV1 { len, cmd, mut offset, mut checksum } => {
                self.in_buf[offset] = c;
                checksum ^= c;
                offset += 1;
                if offset == len {
                    self.cmd_msp = u16::from(cmd);
                    MspPacketState::ChecksumV1 { checksum }
                } else {
                    MspPacketState::PayloadV1 { len, cmd, offset, checksum }
                }
            }
            MspPacketState::HeaderV2 { version, mut offset, mut checksum1, mut checksum2 } => {
                self.in_buf[offset] = c;
                if version == MspVersion::V2overV1 {
                    checksum1 ^= c;
                }
                checksum2 = crc8_dvb_s2(checksum2, c);
                offset += 1;

                // Header size is 5 bytes.
                // V2Native: offset 0..5 (ends at 5)
                // V2overV1: offset 2..7 (ends at 7)
                //let header_len = 5;
                let start_index = if version == MspVersion::V2overV1 { 2 } else { 0 };
                let header_end = start_index + 5;

                if offset == header_end {
                    // Use the start_index to find the V2 fields
                    let flags = self.in_buf[start_index];
                    let cmd = u16::from_le_bytes([self.in_buf[start_index + 1], self.in_buf[start_index + 2]]);
                    let size =
                        u16::from_le_bytes([self.in_buf[start_index + 3], self.in_buf[start_index + 4]]) as usize;
                    if size > Self::INBUF_SIZE {
                        MspPacketState::Idle // <--- This is where your code was tripping!
                    } else {
                        MspPacketState::PayloadV2 {
                            version,
                            len: size,
                            cmd,
                            flags,
                            offset: 0, // Reset to 0 to start filling payload
                            checksum1,
                            checksum2,
                        }
                    }
                } else {
                    MspPacketState::HeaderV2 { version, offset, checksum1, checksum2 }
                }
            }
            MspPacketState::PayloadV2 { version, len, cmd, flags, mut offset, mut checksum1, mut checksum2 } => {
                self.in_buf[offset] = c;
                if version == MspVersion::V2overV1 {
                    checksum1 ^= c;
                }
                checksum2 = crc8_dvb_s2(checksum2, c);
                offset += 1;
                if offset == len {
                    self.cmd_msp = cmd;
                    self.cmd_flags = flags;
                    MspPacketState::ChecksumV2 { version, checksum1, checksum2 }
                } else {
                    MspPacketState::PayloadV2 { version, len, cmd, flags, offset, checksum1, checksum2 }
                }
            }
            MspPacketState::ChecksumV1 { checksum } => {
                self.checksum1 = checksum;
                if checksum == c { MspPacketState::CommandReceived } else { MspPacketState::Idle }
            }
            MspPacketState::ChecksumV2 { version, mut checksum1, checksum2 } => {
                self.checksum2 = checksum2;
                if version == MspVersion::V2overV1 {
                    // V2 over V1 has an extra CRC step
                    if checksum2 == c {
                        // XOR the CRC byte into the V1 sum before moving to the final check
                        checksum1 ^= c;
                        MspPacketState::ChecksumV1 { checksum: checksum1 }
                    } else {
                        MspPacketState::Idle
                    }
                } else {
                    if checksum2 == c { MspPacketState::CommandReceived } else { MspPacketState::Idle }
                }
            }
        }
    }

    pub fn serialize_packet(
        version: MspVersion,
        packet_type: MspPacketType,
        cmd: u16,
        flags: u8,
        payload: &[u8],
        dst: &mut [u8], // Provide a buffer from the caller
    ) -> Result<usize, MspError> {
        let mut offset = 0;

        // Helper to push bytes safely into fixed slice
        let push = |b: u8, dst: &mut [u8], off: &mut usize| -> Result<(), MspError> {
            if *off >= dst.len() {
                return Err(MspError::BufferTooSmall);
            }
            dst[*off] = b;
            *off += 1;
            Ok(())
        };

        match version {
            MspVersion::V1 => {
                push(b'$', dst, &mut offset)?;
                push(b'M', dst, &mut offset)?;
                push(if packet_type == MspPacketType::Command { b'<' } else { b'>' }, dst, &mut offset)?;

                #[allow(clippy::cast_possible_truncation)]
                let size = payload.len() as u8;
                #[allow(clippy::cast_possible_truncation)]
                let cmd_u8 = cmd as u8;
                let mut xor = size ^ cmd_u8;

                push(size, dst, &mut offset)?;
                push(cmd_u8, dst, &mut offset)?;

                for &byte in payload {
                    push(byte, dst, &mut offset)?;
                    xor ^= byte;
                }
                push(xor, dst, &mut offset)?;
            }

            MspVersion::V2 => {
                push(b'$', dst, &mut offset)?;
                push(b'X', dst, &mut offset)?;
                push(if packet_type == MspPacketType::Command { b'<' } else { b'>' }, dst, &mut offset)?;

                let mut crc = 0u8;
                #[allow(clippy::cast_possible_truncation)]
                let size = payload.len() as u16;

                // Nested helper to push and update CRC
                let push_v2 = |b: u8, dst: &mut [u8], off: &mut usize, c: &mut u8| -> Result<(), MspError> {
                    push(b, dst, off)?;
                    *c = crc8_dvb_s2(*c, b);
                    Ok(())
                };

                push_v2(flags, dst, &mut offset, &mut crc)?;
                push_v2((cmd & 0xFF) as u8, dst, &mut offset, &mut crc)?;
                push_v2((cmd >> 8) as u8, dst, &mut offset, &mut crc)?;
                push_v2((size & 0xFF) as u8, dst, &mut offset, &mut crc)?;
                push_v2((size >> 8) as u8, dst, &mut offset, &mut crc)?;

                for &byte in payload {
                    push_v2(byte, dst, &mut offset, &mut crc)?;
                }
                push(crc, dst, &mut offset)?;
            }

            MspVersion::V2overV1 => {
                push(b'$', dst, &mut offset)?;
                push(b'M', dst, &mut offset)?;
                push(if packet_type == MspPacketType::Command { b'<' } else { b'>' }, dst, &mut offset)?;

                let v2_payload_len = 5 + payload.len() + 1; // Hdr(5) + Data + CRC(1)
                #[allow(clippy::cast_possible_truncation)]
                let v1_size = v2_payload_len as u8;
                let v1_cmd = 255u8;
                let mut xor = v1_size ^ v1_cmd;

                push(v1_size, dst, &mut offset)?;
                push(v1_cmd, dst, &mut offset)?;

                let mut crc = 0u8;
                #[allow(clippy::cast_possible_truncation)]
                let size_v2 = payload.len() as u16;

                let push_v2ov1 =
                    |b: u8, dst: &mut [u8], off: &mut usize, c: &mut u8, x: &mut u8| -> Result<(), MspError> {
                        push(b, dst, off)?;
                        *c = crc8_dvb_s2(*c, b);
                        *x ^= b;
                        Ok(())
                    };

                push_v2ov1(flags, dst, &mut offset, &mut crc, &mut xor)?;
                push_v2ov1((cmd & 0xFF) as u8, dst, &mut offset, &mut crc, &mut xor)?;
                push_v2ov1((cmd >> 8) as u8, dst, &mut offset, &mut crc, &mut xor)?;
                push_v2ov1((size_v2 & 0xFF) as u8, dst, &mut offset, &mut crc, &mut xor)?;
                push_v2ov1((size_v2 >> 8) as u8, dst, &mut offset, &mut crc, &mut xor)?;

                for &byte in payload {
                    push_v2ov1(byte, dst, &mut offset, &mut crc, &mut xor)?;
                }

                push(crc, dst, &mut offset)?;
                xor ^= crc;
                push(xor, dst, &mut offset)?;
            }
        }

        Ok(offset) // Return number of bytes written
    }
}

#[derive(Debug)]
#[allow(unused)]
pub enum MspError {
    BufferTooSmall,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msp_v1_parsing() {
        let mut stream = MspStream::new();
        let packet: [u8; 7] = [
            b'M', b'<', 2,   // Size
            100, // Cmd
            1, 2,   // Payload
            101, // XOR
        ];

        for &byte in &packet {
            stream.process_received_packet_data(byte);
        }

        assert!(matches!(stream.packet_state, MspPacketState::CommandReceived));
        assert_eq!(stream.cmd_msp, 100);
        assert_eq!(&stream.in_buf[0..2], &[1, 2]);
    }
    #[test]
    fn test_msp_v2_native_parsing() {
        let mut stream = MspStream::new();
        let packet: [u8; 10] = [
            b'X', b'<', 0, // Flags
            2, 1, // Cmd (0x0102)
            2, 0, // Size (2)
            0xAA, 0xBB, // Payload
            19,   // CRC8
        ];

        for &byte in &packet {
            stream.process_received_packet_data(byte);
        }

        assert_eq!(MspPacketState::CommandReceived, stream.packet_state);
        assert_eq!(19, stream.checksum2);
        assert_eq!(stream.cmd_msp, 0x0102);
        assert_eq!(stream.cmd_flags, 0);
        assert_eq!(&stream.in_buf[0..2], &[0xAA, 0xBB]);
    }
    #[test]
    fn test_msp_v2_over_v1_parsing() {
        let mut stream = MspStream::new();

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

        for (i, &byte) in packet.iter().enumerate() {
            stream.process_received_packet_data(byte);

            if i < packet.len() - 1 {
                // Ensure we haven't reset to Idle prematurely
                assert!(
                    !matches!(stream.packet_state, MspPacketState::Idle),
                    "State machine reset to Idle at byte index {i}",
                );
            }
        }

        // Final Verification
        assert_eq!(stream.packet_state, MspPacketState::CommandReceived);
        assert_eq!(244, stream.checksum1);
        assert_eq!(19, stream.checksum2);
        assert_eq!(stream.cmd_msp, 0x0102);
        assert_eq!(stream.cmd_flags, 0);
        assert_eq!(&stream.in_buf[0..2], &[0xAA, 0xBB]);
    }
    #[test]
    fn test_serialize_msp_v1_request() {
        let mut buf = [0u8; 32];

        // Example: MSP_IDENT (Cmd 100) request with no payload
        let size = MspStream::serialize_packet(
            MspVersion::V1,
            MspPacketType::Command,
            100,
            0,
            &[], // Empty payload
            &mut buf,
        )
        .expect("Serialization failed");

        // Expected: $M< (3) + Size (0) + Cmd (100) + XOR (100)
        // Checksum: 0 ^ 100 = 100
        let expected = [b'$', b'M', b'<', 0, 100, 100];
        assert_eq!(&buf[..size], &expected);
    }

    #[test]
    fn test_serialize_msp_v1_with_payload() {
        let mut buf = [0u8; 32];

        // Example: Command 100 with payload [1, 2]
        let payload = [1, 2];
        let size = MspStream::serialize_packet(MspVersion::V1, MspPacketType::Command, 100, 0, &payload, &mut buf)
            .expect("Serialization failed");

        // Expected: $M< (3) + Size (2) + Cmd (100) + Payload (1, 2) + XOR
        // Checksum: 2 ^ 100 ^ 1 ^ 2 = 101
        let expected = [b'$', b'M', b'<', 2, 100, 1, 2, 101];
        assert_eq!(&buf[..size], &expected);
    }
    #[test]
    fn test_serialize_v2_over_v1() {
        let mut buf = [0u8; 64];
        let payload = [0xAA, 0xBB];

        let size = MspStream::serialize_packet(
            MspVersion::V2overV1,
            MspPacketType::Command,
            0x0102, // Cmd
            0,      // Flags
            &payload,
            &mut buf,
        )
        .expect("Serialization failed");

        // Breakdown of the expected 13 bytes:
        // [0..3]   Header: $M<
        // [3]      V1 Size: 8
        // [4]      V1 Cmd: 255
        // [5..10]  V2 Header: 0, 2, 1, 2, 0 (Flags, CmdL, CmdH, SizeL, SizeH)
        // [10..12] V2 Payload: 0xAA, 0xBB
        // [12]     V2 CRC8: 19
        // [13]     V1 XOR: 244
        let expected = [
            b'$', b'M', b'<', 8, 255, // V1 Wrapper
            0, 2, 1, 2, 0, // V2 Header
            0xAA, 0xBB, // V2 Data
            19,   // V2 CRC
            244,  // V1 XOR
        ];

        assert_eq!(size, 14); // 3 (hdr) + 11 (data/checksums) = 14 total bytes
        assert_eq!(&buf[..size], &expected);
    }
}
