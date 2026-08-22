macro_rules! make_frame {
    () => {
        pub fn make_frame(self) -> [u8; Self::FRAME_LEN] {
            let mut frame = [0u8; Self::FRAME_LEN];

            frame[0..4].copy_from_slice(&[UbxParser::SYNC_BYTE_1, UbxParser::SYNC_BYTE_2, Self::CLASS as u8, Self::ID]);

            frame[4..6].copy_from_slice(&Self::PAYLOAD_LEN_U16.to_le_bytes());

            frame[6..6 + Self::PAYLOAD_LEN].copy_from_slice(&self.make_payload());

            // UBX Fletcher checksum covers class, ID, length and payload.
            let mut checksum = [0u8; 2];
            for &byte in &frame[2..Self::FRAME_LEN - 2] {
                checksum[0] = checksum[0].wrapping_add(byte);
                checksum[1] = checksum[1].wrapping_add(checksum[0]);
            }

            frame[Self::FRAME_LEN - 2..Self::FRAME_LEN].copy_from_slice(&checksum);

            frame
        }
    };
}
