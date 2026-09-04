/// DVB-S2 CRC-8.
///
/// Polynomial:      0xD5
/// Initial value:   0x00
/// Input reflected: false
/// Result reflected:false
/// `XorOut`:        0x00.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CrcDvbS2;

impl CrcDvbS2 {
    #[allow(unused)]
    #[inline]
    #[must_use]
    pub fn calculate(data: &[u8]) -> u8 {
        let mut crc = 0;
        for &byte in data {
            crc = Self::LOOKUP[usize::from(crc ^ byte)];
        }
        crc
    }
    #[inline]
    #[must_use]
    pub fn update(crc: u8, byte: u8) -> u8 {
        Self::LOOKUP[usize::from(crc ^ byte)]
    }
}

impl CrcDvbS2 {
    /// DVB-S2 polynomial.
    const POLYNOMIAL: u8 = 0xD5;
    /// Pre-computed 256-entry lookup table, generated at compile time.
    /// Initial Value: 0x00, Input Reflected: False, Result Reflected: False.
    const LOOKUP: [u8; 256] = Self::make_table();

    const fn table_entry(mut byte: u8) -> u8 {
        let mut ii = 0;
        while ii < 8 {
            if byte & 0x80 != 0 {
                byte = (byte << 1) ^ Self::POLYNOMIAL;
            } else {
                byte <<= 1;
            }
            ii += 1;
        }
        byte
    }

    const fn make_table() -> [u8; 256] {
        let mut table = [0u8; 256];
        let mut ii = 0;
        #[allow(clippy::cast_possible_truncation)]
        while ii < 256 {
            table[ii] = Self::table_entry(ii as u8);
            ii += 1;
        }
        table
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<CrcDvbS2>();
    }
}

#[cfg(test)]
mod crc_tests {
    use super::*;

    #[must_use]
    pub fn calculate_crc(data: &[u8]) -> u8 {
        let mut crc = 0;
        for &byte in data {
            crc ^= byte;
            for _ in 0..8 {
                if (crc & 0x80) == 0 {
                    crc <<= 1;
                } else {
                    crc = (crc << 1) ^ CrcDvbS2::POLYNOMIAL;
                }
            }
        }
        crc
    }
    #[test]
    fn check_value() {
        assert_eq!(0xBC, CrcDvbS2::calculate(b"123456789"));
    }
    #[test]
    fn table_crc_should_match_bit_by_bit_crc_outputs() {
        // Arrange
        let test_payloads: [&[u8]; 4] =
            [b"", b"hello", b"data_stream_xyz_12345", &[0x01, 0x02, 0x03, 0x04, 0x05, 0xFF, 0x00]];

        // Act & Assert
        for payload in &test_payloads {
            let bit_by_bit_result = calculate_crc(payload);
            let table_result = CrcDvbS2::calculate(payload);

            assert_eq!(table_result, bit_by_bit_result, "CRC mismatch for payload: {payload:?}");
        }
    }
}
