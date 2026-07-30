#![cfg(feature = "osd")]

/// A lightweight, ultra-fast writer cursor that replaces `core::fmt::Write`.
/// Used by `OsdElement::write_custom`.
#[derive(Debug, Default, PartialEq)]
pub struct OsdBufferCursor<'a> {
    pub buf: &'a mut [u8],
    pub pos: usize,
}

impl OsdBufferCursor<'_> {
    /// Maximum decimal digits required to hold any 32-bit unsigned integer string.
    pub const U32_MAX_DIGITS: usize = 10;

    /// Appends a raw static byte sequence safely.
    #[allow(unused)]
    pub fn append_bytes(&mut self, bytes: &[u8]) {
        let remaining = self.buf.len() - self.pos;
        let to_copy = bytes.len().min(remaining);

        if to_copy > 0 {
            self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
            self.pos += to_copy;
        }
    }

    /// Append a static string slice.
    #[allow(unused)]
    pub fn append_str(&mut self, text: &str) {
        self.append_bytes(text.as_bytes());
    }

    /// Optimized, division-based integer-to-ASCII formatter.
    #[allow(unused)]
    pub fn append_u32(&mut self, mut value: u32) {
        if value == 0 {
            self.append_bytes(b"0");
            return;
        }

        let mut temp = [0u8; Self::U32_MAX_DIGITS];
        let mut ii = 0;

        while value > 0 && ii < temp.len() {
            temp[ii] = b'0' + (value % 10) as u8;
            value /= 10;
            ii += 1;
        }

        // Copy backwards into the main buffer to fix character order
        let remaining = self.buf.len() - self.pos;
        let to_copy = ii.min(remaining);

        for offset in 0..to_copy {
            self.buf[self.pos + offset] = temp[ii - 1 - offset];
        }
        self.pos += to_copy;
    }

    /// Appends a string right-aligned within a field of a specified width.
    /// If the string is longer than the field width, it will be left-truncated.
    #[allow(unused)]
    pub fn append_str_right_aligned(&mut self, text: &str, field_width: usize) {
        let bytes = text.as_bytes();
        let remaining = self.buf.len() - self.pos;

        // Ensure we don't exceed the requested field width or remaining buffer space
        let max_width = field_width.min(remaining);
        if max_width == 0 {
            return;
        }

        let text_len = bytes.len();

        if text_len >= max_width {
            // Text is too long for the field: take the tail of the string
            let start_idx = text_len - max_width;
            let to_copy = &bytes[start_idx..];
            self.buf[self.pos..self.pos + max_width].copy_from_slice(to_copy);
            self.pos += max_width;
        } else {
            // Text is shorter: fill the left side with padding spaces
            let spaces_count = max_width - text_len;
            self.buf[self.pos..self.pos + spaces_count].fill(b' ');
            self.pos += spaces_count;

            // Copy the actual text on the right side
            self.buf[self.pos..self.pos + text_len].copy_from_slice(bytes);
            self.pos += text_len;
        }
    }

    /// Appends an integer right-aligned within a field of a specified width.
    /// You can specify whether to pad the empty left space with zeroes ('0') or spaces (' ').
    #[allow(unused)]
    pub fn append_u32_right_aligned(&mut self, mut value: u32, field_width: usize, pad_with_zero: bool) {
        let remaining = self.buf.len() - self.pos;
        let max_width = field_width.min(remaining).min(Self::U32_MAX_DIGITS);
        if max_width == 0 {
            return;
        }

        // Write digits into a temporary array in reverse order
        let mut buf = [0u8; Self::U32_MAX_DIGITS];
        let mut digit_count = 0;

        if value == 0 {
            buf[0] = b'0';
            digit_count = 1;
        } else {
            while value > 0 && digit_count < buf.len() {
                buf[digit_count] = b'0' + (value % 10) as u8;
                value /= 10;
                digit_count += 1;
            }
        }

        // Determine what padding character to use
        let pad_char = if pad_with_zero { b'0' } else { b' ' };

        if digit_count >= max_width {
            // Number has more digits than the field width: truncate the leading digits
            for offset in 0..max_width {
                // Read backwards from the end of the required visible tail slice
                self.buf[self.pos + offset] = buf[max_width - 1 - offset];
            }
            self.pos += max_width;
        } else {
            // Number is smaller than field width: inject padding first
            let padding_needed = max_width - digit_count;
            self.buf[self.pos..self.pos + padding_needed].fill(pad_char);
            self.pos += padding_needed;

            // Copy the numbers into the remaining right-hand slots
            for offset in 0..digit_count {
                self.buf[self.pos + offset] = buf[digit_count - 1 - offset];
            }
            self.pos += digit_count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn _is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    fn is_partial<T: Sized + Send + Sync + Unpin + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_partial::<OsdBufferCursor>();
    }
}
