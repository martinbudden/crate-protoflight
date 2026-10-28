#![allow(unused)]
use crate::multiwii_serial_protocol::msp_parser::MspPacket;

use super::crc_dvb_s2::CrcDvbS2;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
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
    packet_type: MspPacketType,
    msp_version: MspVersion,
    pub cmd_msp: u16,
    pub offset: u16,
    pub data_size: u16,
    pub cmd_flags: u8,
    pub checksum1: u8,
    pub checksum2: u8,
}

#[allow(unused)]
impl MspStream {
    //const JUMBO_FRAME_SIZE_LIMIT: usize = 255;

    //const MSP_EVALUATE_NON_MSP_DATA: u8 = 0;
    //const MSP_SKIP_NON_MSP_DATA: u8 = 1;
    //const MSP_HEADER_LENGTH: usize = 3;

    //const MSP_MAX_HEADER_SIZE: usize = 9;
}

impl MspStream {
    pub const fn new() -> Self {
        Self {
            //pending_request: MspPendingSystemRequest::None,
            //stream_state: MspStreamState::Idle,
            packet_type: MspPacketType::Command,
            msp_version: MspVersion::V1,
            cmd_msp: 0,
            offset: 0,
            data_size: 0,
            cmd_flags: 0,
            checksum1: 0,
            checksum2: 0,
        }
    }
}

impl Default for MspStream {
    fn default() -> Self {
        Self::new()
    }
}

impl MspStream {
    #[allow(unused)]
    pub fn msp_encode(
        msp_version: MspVersion,
        packet_type: MspPacketType,
        cmd: u16,
        flags: u8,
        payload: &[u8],
        output_buf: &mut [u8], // caller provided buffer
    ) -> Result<usize, MspError> {
        let mut offset = 0;

        // Helper to push bytes safely into fixed slice
        let push = |b: u8, output_buf: &mut [u8], offset: &mut usize| -> Result<(), MspError> {
            if *offset >= output_buf.len() {
                return Err(MspError::BufferTooSmall);
            }
            output_buf[*offset] = b;
            *offset += 1;
            Ok(())
        };

        match msp_version {
            MspVersion::V1 => {
                push(b'$', output_buf, &mut offset)?;
                push(b'M', output_buf, &mut offset)?;
                push(if packet_type == MspPacketType::Command { b'<' } else { b'>' }, output_buf, &mut offset)?;

                #[allow(clippy::cast_possible_truncation)]
                let size = payload.len() as u8;
                push(size, output_buf, &mut offset)?;

                #[allow(clippy::cast_possible_truncation)]
                let cmd_u8 = cmd as u8;
                push(cmd_u8, output_buf, &mut offset)?;

                let mut checksum = size ^ cmd_u8;
                for &byte in payload {
                    push(byte, output_buf, &mut offset)?;
                    checksum ^= byte;
                }
                push(checksum, output_buf, &mut offset)?;
            }

            MspVersion::V2 => {
                push(b'$', output_buf, &mut offset)?;
                push(b'X', output_buf, &mut offset)?;
                push(if packet_type == MspPacketType::Command { b'<' } else { b'>' }, output_buf, &mut offset)?;

                let mut crc = 0u8;
                #[allow(clippy::cast_possible_truncation)]
                let size = payload.len() as u16;

                // Nested helper to push and update CRC
                let push_v2 = |b: u8, dst: &mut [u8], offset: &mut usize, crc: &mut u8| -> Result<(), MspError> {
                    push(b, dst, offset)?;
                    *crc = CrcDvbS2::update(*crc, b);
                    Ok(())
                };

                push_v2(flags, output_buf, &mut offset, &mut crc)?;

                let cmd_bytes = cmd.to_be_bytes();
                push_v2(cmd_bytes[0], output_buf, &mut offset, &mut crc)?;
                push_v2(cmd_bytes[1], output_buf, &mut offset, &mut crc)?;

                let size_bytes = size.to_be_bytes();
                push_v2(size_bytes[0], output_buf, &mut offset, &mut crc)?;
                push_v2(size_bytes[1], output_buf, &mut offset, &mut crc)?;

                for &byte in payload {
                    push_v2(byte, output_buf, &mut offset, &mut crc)?;
                }
                push(crc, output_buf, &mut offset)?;
            }

            MspVersion::V2overV1 => {
                push(b'$', output_buf, &mut offset)?;
                push(b'M', output_buf, &mut offset)?;
                push(if packet_type == MspPacketType::Command { b'<' } else { b'>' }, output_buf, &mut offset)?;

                let v2_payload_len = 5 + payload.len() + 1; // Hdr(5) + Data + CRC(1)
                #[allow(clippy::cast_possible_truncation)]
                let v1_size = v2_payload_len as u8;
                let v1_cmd = 255u8;
                let mut xor = v1_size ^ v1_cmd;

                push(v1_size, output_buf, &mut offset)?;
                push(v1_cmd, output_buf, &mut offset)?;

                let mut crc = 0u8;
                #[allow(clippy::cast_possible_truncation)]
                let size_v2 = payload.len() as u16;

                let push_v2_over_v1 =
                    |b: u8, dst: &mut [u8], off: &mut usize, c: &mut u8, x: &mut u8| -> Result<(), MspError> {
                        push(b, dst, off)?;
                        *c = CrcDvbS2::update(*c, b);
                        *x ^= b;
                        Ok(())
                    };

                push_v2_over_v1(flags, output_buf, &mut offset, &mut crc, &mut xor)?;
                push_v2_over_v1((cmd & 0xFF) as u8, output_buf, &mut offset, &mut crc, &mut xor)?;
                push_v2_over_v1((cmd >> 8) as u8, output_buf, &mut offset, &mut crc, &mut xor)?;
                push_v2_over_v1((size_v2 & 0xFF) as u8, output_buf, &mut offset, &mut crc, &mut xor)?;
                push_v2_over_v1((size_v2 >> 8) as u8, output_buf, &mut offset, &mut crc, &mut xor)?;

                for &byte in payload {
                    push_v2_over_v1(byte, output_buf, &mut offset, &mut crc, &mut xor)?;
                }

                push(crc, output_buf, &mut offset)?;
                xor ^= crc;
                push(xor, output_buf, &mut offset)?;
            }
        }

        Ok(offset) // Return number of bytes written
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum MspError {
    #[default]
    BufferTooSmall,
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<MspVersion>();
        is_full::<MspPacketType>();
        is_full::<MspStream>();
        is_full::<MspError>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const MSP_IDENT: u16 = 100;
    const RAW_IMU: u16 = 102;

    #[test]
    fn test_serialize_msp_v1_request() {
        let mut output_buf = [0u8; 32];

        // Example: MSP_IDENT (Cmd 100) request with no payload
        let result = MspStream::msp_encode(
            MspVersion::V1,
            MspPacketType::Command,
            100,
            0,
            &[], // Empty payload
            &mut output_buf,
        );
        assert!(result.is_ok());
        if let Ok(size) = result {
            // Expected: $M< (3) + Size (0) + Cmd (100) + XOR (100)
            // Checksum: 0 ^ 100 = 100
            let expected = [b'$', b'M', b'<', 0, 100, 100];
            assert_eq!(&output_buf[..size], &expected);
        }
    }

    #[test]
    fn test_serialize_msp_v1_with_payload() {
        let mut output_buf = [0u8; 32];

        // Example: Command 100 with payload [1, 2]
        let payload = [1u8, 2u8];
        let result =
            MspStream::msp_encode(MspVersion::V1, MspPacketType::Command, MSP_IDENT, 0, &payload, &mut output_buf);

        assert!(result.is_ok());
        if let Ok(size) = result {
            // Expected: $M< (3) + Size (2) + Cmd (100) + Payload (1, 2) + XOR
            // Checksum: 2 ^ 100 ^ 1 ^ 2 = 101
            let expected = [b'$', b'M', b'<', 2, 100, 1, 2, 101];
            assert_eq!(&output_buf[..size], &expected);
        }
    }
    #[test]
    fn test_serialize_v2_over_v1() {
        let mut output_buf = [0u8; 64];

        let payload = [0xAAu8, 0xBBu8];

        let result = MspStream::msp_encode(
            MspVersion::V2overV1,
            MspPacketType::Command,
            0x0102, // Cmd
            0,      // Flags
            &payload,
            &mut output_buf,
        );

        assert!(result.is_ok());
        if let Ok(size) = result {
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
            assert_eq!(&output_buf[..size], &expected);
        }
    }
}
