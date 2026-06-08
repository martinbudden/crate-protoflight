#![cfg(feature = "osd")]
#![allow(unused)]
use crate::{
    display::{Display, DisplayClear, DisplayPortLayer, DisplayPortSeverity},
    osd::{OsdElementsConfig, display::OsdDrawContext, elements_draw::OsdElementId},
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OsdElementType {
    #[default]
    Type1,
    Type2,
    Type3,
    Type4,
}

impl OsdElementType {
    fn from(v: u16) -> Self {
        match v {
            1 => Self::Type2,
            2 => Self::Type3,
            3 => Self::Type4,
            _ => Self::Type1, // default to Type1
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OsdElement {
    pub buf: [u8; Self::BUFFER_LENGTH],
    pub element_type: OsdElementType,
    pub id: OsdElementId,
    pub pos_x: u8,
    pub pos_y: u8,
    pub offset_x: u8,
    pub offset_y: u8,
    pub attr: u8,
    pub rendered: bool,
    pub draw_element: bool,
    pub horizon_x: i32,
}

impl OsdElement {
    pub const BUFFER_LENGTH: usize = 32;

    pub const fn new() -> Self {
        Self {
            buf: [0u8; Self::BUFFER_LENGTH],
            element_type: OsdElementType::Type1,
            id: OsdElementId::Altitude,
            pos_x: 0,
            pos_y: 0,
            offset_x: 0,
            offset_y: 0,
            attr: 0,
            rendered: false,
            draw_element: false,
            horizon_x: -4,
        }
    }
}

impl Default for OsdElement {
    fn default() -> Self {
        Self::new()
    }
}

impl OsdElement {
    /// Overwrites the buffer completely with a static string and fills the rest with 0.
    pub fn set_text(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let len = bytes.len().min(Self::BUFFER_LENGTH);

        self.buf[..len].copy_from_slice(&bytes[..len]);
        self.buf[len..].fill(0);
    }

    /// Flexible multi-part writer that allows concatenating text and numbers manually.
    /// Returns the exact number of bytes written.
    pub fn write_custom<F>(&mut self, write_logic: F) -> usize
    where
        F: FnOnce(&mut OsdBufferCursor),
    {
        // 1. Clear the element's internal array buffer
        self.buf.fill(0);

        // 2. Create an ephemeral tracking cursor over our data slice
        let mut cursor = OsdBufferCursor { buf: &mut self.buf, pos: 0 };

        // 3. Execute the writing logic steps
        write_logic(&mut cursor);

        cursor.pos
    }
}

/// A lightweight, ultra-fast writer cursor that replaces `core::fmt::Write`.
pub struct OsdBufferCursor<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl OsdBufferCursor<'_> {
    /// Maximum decimal digits required to hold any 32-bit unsigned integer string.
    pub const U32_MAX_DIGITS: usize = 10;

    /// Appends a raw static byte sequence safely.
    pub fn append_bytes(&mut self, bytes: &[u8]) {
        let remaining = self.buf.len() - self.pos;
        let to_copy = bytes.len().min(remaining);

        if to_copy > 0 {
            self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
            self.pos += to_copy;
        }
    }

    /// Append a static string slice.
    pub fn append_str(&mut self, text: &str) {
        self.append_bytes(text.as_bytes());
    }

    /// Optimized, division-based integer-to-ASCII formatter.
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
    pub fn append_u32_right_aligned(&mut self, mut value: u32, field_width: usize, pad_with_zero: bool) {
        let remaining = self.buf.len() - self.pos;
        let max_width = field_width.min(remaining).min(Self::U32_MAX_DIGITS);
        if max_width == 0 {
            return;
        }

        // 1. Generate digits into a temporary array in reverse order
        let mut temp = [0u8; Self::U32_MAX_DIGITS];
        let mut digit_count = 0;

        if value == 0 {
            temp[0] = b'0';
            digit_count = 1;
        } else {
            while value > 0 && digit_count < temp.len() {
                temp[digit_count] = b'0' + (value % 10) as u8;
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
                self.buf[self.pos + offset] = temp[max_width - 1 - offset];
            }
            self.pos += max_width;
        } else {
            // Number is smaller than field width: inject padding first
            let padding_needed = max_width - digit_count;
            self.buf[self.pos..self.pos + padding_needed].fill(pad_char);
            self.pos += padding_needed;

            // Copy the numbers into the remaining right-hand slots
            for offset in 0..digit_count {
                self.buf[self.pos + offset] = temp[digit_count - 1 - offset];
            }
            self.pos += digit_count;
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OsdElements {
    config: OsdElementsConfig,
    pub active_element: OsdElement,
    active_elements: [OsdElementId; Self::COUNT],
    active_element_count: usize,
    active_element_index: usize,
    profile: u8,
    // TODO: change this to state variable
    display_pending_foreground: bool,
    display_pending_background: bool,
    background_rendered: bool,
    background_layer_supported: bool,
    pub roll_angle_degrees: f32,
    pub pitch_angle_degrees: f32,
    pub yaw_angle_degrees: f32,
}

impl OsdElements {
    pub const fn new() -> Self {
        Self {
            config: OsdElementsConfig::new(),
            active_element: OsdElement::new(),
            active_elements: [OsdElementId::Rssi; Self::COUNT],
            active_element_count: 0,
            active_element_index: 0,
            profile: 0,
            display_pending_foreground: false,
            display_pending_background: false,
            background_rendered: false,
            background_layer_supported: false,
            roll_angle_degrees: 0.0,
            pitch_angle_degrees: 0.0,
            yaw_angle_degrees: 0.0,
        }
    }
}

impl Default for OsdElements {
    fn default() -> Self {
        Self::new()
    }
}

#[rustfmt::skip]
impl OsdElements {
    pub const COUNT: usize = 32;
    pub const ELEMENT_BITS_POS: u16 = 14;
    pub const PROFILE_BITS_POS: u16 = 12;
    pub const XY_POSITION_BITS: u16 = 6; // 6 bits gives a range 0-63
    pub const ELEMENT_TYPE_MASK: u16 = 0b_1100_0000_0000_0000; // bits 14-15
    pub const PROFILE_MASK:      u16 = 0b_0011_0000_0000_0000;
    pub const _Y_POSITION_MASK:   u16 = 0b_0000_1111_1100_0000;
    pub const X_POSITION_MASK:   u16 = 0b_0000_0000_0011_1111;
}

#[allow(unused)]
#[allow(clippy::unused_self)]
impl OsdElements {
    pub fn element_type(x: u16) -> OsdElementType {
        OsdElementType::from((x & Self::ELEMENT_TYPE_MASK) >> Self::ELEMENT_BITS_POS)
    }

    pub fn profile_flag(x: u16) -> u16 {
        1 << (x - 1 + Self::PROFILE_BITS_POS)
    }

    pub fn set_profile(&mut self, profile: u8) {
        self.profile = profile.clamp(0, 1);
    }

    pub fn element_visible(value: u16, profile: u8) -> bool {
        ((value & Self::PROFILE_MASK) >> Self::PROFILE_BITS_POS) & (1 << profile) == 0
    }

    pub fn pos_x(x: u16) -> u8 {
        (x & Self::X_POSITION_MASK) as u8
    }

    pub fn pos_y(x: u16) -> u8 {
        ((x >> Self::XY_POSITION_BITS) & Self::X_POSITION_MASK) as u8
    }

    pub fn pos(x: u16, y: u16) -> u16 {
        (x & Self::X_POSITION_MASK) | ((y & Self::X_POSITION_MASK) << Self::XY_POSITION_BITS)
    }

    pub fn config(&self) -> OsdElementsConfig {
        self.config
    }

    pub fn set_config(&mut self, config: OsdElementsConfig) {
        self.config = config;
    }

    pub fn add_active_element(&mut self, element: OsdElementId) {
        if Self::element_visible(self.config.positions[element as usize], self.profile) {
            self.active_elements[self.active_element_count] = element;
            self.active_element_count += 1;
        }
    }

    pub fn is_render_pending(&self) -> bool {
        self.display_pending_foreground | self.display_pending_background
    }

    pub fn active_element_index(&self) -> usize {
        self.active_element_index
    }

    pub fn active_element_count(&self) -> usize {
        self.active_element_count
    }

    pub fn draw_next_active_element<D: Display>(&mut self, draw_context: &OsdDrawContext<D>) -> bool {
        if self.active_element_index >= self.active_element_count {
            self.active_element_index = 0;
            return false;
        }

        let element_id = self.active_elements[self.active_element_index];

        if !self.background_layer_supported && !self.background_rendered {
            //  && DrawBackgroundFunctions[element]
            // If the background layer isn't supported then we
            // have to draw the element's static layer as well.
            self.background_rendered = self.draw_element_background_by_id(element_id, draw_context);
            // After the background always come back to check for foreground
            return true;
        }

        if self.draw_element_by_id(element_id, draw_context) {
            // If rendering is complete then advance to the next element
            // Prepare to render the background of the next element
            self.background_rendered = false;
            self.active_element_index += 1;
            if self.active_element_index >= self.active_element_count {
                self.active_element_index = 0;
                return false;
            }
        }
        true
    }

    pub fn display_active_element<D: Display>(&mut self, draw_context: &mut OsdDrawContext<D>) -> bool {
        if self.active_element_index >= self.active_element_count {
            return false;
        }
        // If there's a previously drawn background string to be displayed, do that
        if self.display_pending_background {
            _ = draw_context.display_port.write_string(
                self.active_element.pos_x + self.active_element.offset_x,
                self.active_element.pos_y + self.active_element.offset_y,
                &self.active_element.buf,
                self.active_element.attr,
            );
            self.active_element.buf[0] = 0;
            self.display_pending_background = false;
            return self.display_pending_foreground;
        }
        // If there's a previously drawn foreground string to be displayed, do that
        if self.display_pending_foreground {
            _ = draw_context.display_port.write_string(
                self.active_element.pos_x + self.active_element.offset_x,
                self.active_element.pos_y + self.active_element.offset_y,
                &self.active_element.buf,
                self.active_element.attr,
            );
            self.active_element.buf[0] = 0;
            self.display_pending_foreground = false;
        }
        false
    }
    pub fn draw_spec(&self) -> bool {
        true
    }

    pub fn draw_element_by_id<D: Display>(
        &mut self,
        element_id: OsdElementId,
        draw_context: &OsdDrawContext<D>,
    ) -> bool {
        // By default mark the element as rendered in case it's in the off blink state

        /*if (!DrawFunctions[element_index]) {
            // Element has no drawing function
            return true;
        }
        if (!ctx.display_port.get_use_device_blink() && _blink_bits[element_index]) {
            return true;
        }*/

        let position = self.config.positions[element_id as usize];
        self.active_element = OsdElement {
            buf: [0u8; OsdElement::BUFFER_LENGTH],
            element_type: Self::element_type(position),
            id: element_id,
            pos_x: Self::pos_x(position),
            pos_y: Self::pos_y(position),
            offset_x: 0,
            offset_y: 0,
            attr: DisplayPortSeverity::Normal as u8,
            rendered: true,
            draw_element: true,
            horizon_x: -4,
        };

        // TODO: need to check drawing of SYS elements
        // Call the element drawing function
        if self.draw_element(draw_context) {
            self.display_pending_foreground = true;
        }

        self.active_element.rendered
    }

    pub fn draw_element_background_by_id<D: Display>(
        &mut self,
        element_id: OsdElementId,
        draw_context: &OsdDrawContext<D>,
    ) -> bool {
        /*if (!DrawBackgroundFunctions[element_index]) {
            return true;
        }*/
        self.active_element = OsdElement {
            buf: [0u8; OsdElement::BUFFER_LENGTH],
            element_type: Self::element_type(self.config.positions[element_id as usize]),
            id: element_id,
            pos_x: Self::pos_x(self.config.positions[element_id as usize]),
            pos_y: Self::pos_y(self.config.positions[element_id as usize]),
            offset_x: 0,
            offset_y: 0,
            attr: DisplayPortSeverity::Normal as u8,
            rendered: true,
            draw_element: true,
            horizon_x: -4,
        };

        if self.draw_element_background(draw_context) {
            self.display_pending_background = true;
        }

        self.active_element.rendered
    }

    pub fn draw_active_elements_background<D: Display>(&mut self, draw_context: &mut OsdDrawContext<D>) {
        if self.background_layer_supported {
            draw_context.display_port.layer_select(DisplayPortLayer::Background);
            draw_context.display_port.clear_screen(DisplayClear::Wait);
            for element_id in self.active_elements {
                while !self.draw_element_background_by_id(element_id, draw_context) {}
            }
            draw_context.display_port.layer_select(DisplayPortLayer::Foreground);
        }
    }

    pub fn update_attitude(&mut self, roll_angle_degrees: f32, pitch_angle_degrees: f32, yaw_angle_degrees: f32) {
        self.roll_angle_degrees = roll_angle_degrees;
        self.pitch_angle_degrees = pitch_angle_degrees;
        self.yaw_angle_degrees = yaw_angle_degrees;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<OsdElement>();
        is_full::<OsdElements>();
        is_full::<OsdElementType>();
    }
}
