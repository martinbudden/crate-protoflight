#![cfg(feature = "osd")]

use crate::{
    sensors::SensorFlags,
    display::{Display, DisplayPortLayer, DisplayPortSeverity},
    osd::{
        OsdElementsConfig,
        display::OsdDrawContext,
        elements_draw::{OSD_ELEMENT_DISPLAY_ORDER, OsdElementId},
        fixed_buf::FixedBuf,
        //osd_buffer_cursor::OsdBufferCursor,
    },
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OsdStickOverlayRenderPhase {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OsdStickCameraFrameRenderPhase {
    #[default]
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OsdElement {
    pub buf: FixedBuf<{ OsdElement::BUFFER_LENGTH }>,
    pub element_type: OsdElementType,
    pub id: OsdElementId,
    pub horizon_x: i32,
    pub pos_x: u8,
    pub pos_y: u8,
    pub offset_x: u8,
    pub offset_y: u8,
    pub attr: DisplayPortSeverity,
    pub rendered: bool,
    pub draw_element: bool,
    pub stick_overlay_render_phase: OsdStickOverlayRenderPhase,
    pub stick_overlay_y: u8,
    pub sidebar_y: i8,
    pub sidebar_render_level: bool,
    pub camera_frame_render_phase: OsdStickCameraFrameRenderPhase,
    pub camera_frame_i: u8,
    pub osd_cap_alarm: i16,
}

impl OsdElement {
    pub const BUFFER_LENGTH: usize = 32;

    pub const fn new() -> Self {
        Self {
            buf: FixedBuf::new(),
            element_type: OsdElementType::Type1,
            id: OsdElementId::Altitude,
            horizon_x: -4,
            pos_x: 0,
            pos_y: 0,
            offset_x: 0,
            offset_y: 0,
            attr: DisplayPortSeverity::Normal,
            rendered: false,
            draw_element: false,
            stick_overlay_render_phase: OsdStickOverlayRenderPhase::Vertical,
            stick_overlay_y: 0,
            sidebar_y: 0,
            sidebar_render_level: false,
            camera_frame_render_phase: OsdStickCameraFrameRenderPhase::Top,
            camera_frame_i: 0,
            osd_cap_alarm: 0,
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
    pub fn write_string(&mut self, string: &str) {
        let bytes = string.as_bytes();
        let len = bytes.len().min(Self::BUFFER_LENGTH);

        self.buf.buf[..len].copy_from_slice(&bytes[..len]);
        self.buf.buf[len..].fill(0);
    }

    pub fn write_slice(&mut self, slice: &[u8]) {
        let len = slice.len().min(Self::BUFFER_LENGTH);

        self.buf.buf[..len].copy_from_slice(&slice[..len]);
        self.buf.buf[len..].fill(0);
    }

    /*/// Flexible multi-part writer that allows concatenating text and numbers manually.
    /// Returns the number of bytes written.
    pub fn write_custom<F>(&mut self, write_logic: F) -> usize
    where
        F: FnOnce(&mut OsdBufferCursor),
    {
        self.buf.buf.fill(0);

        let mut cursor = OsdBufferCursor { buf: &mut self.buf.buf, pos: 0 };

        write_logic(&mut cursor);

        cursor.pos
    }*/
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
    pub roll_angle_degrees: i32,
    pub pitch_angle_degrees: i32,
    pub yaw_angle_degrees: i32,
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
            roll_angle_degrees: 0,
            pitch_angle_degrees: 0,
            yaw_angle_degrees: 0,
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
    pub const _Y_POSITION_MASK:  u16 = 0b_0000_1111_1100_0000;
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

    pub async fn draw_next_active_element<D: Display>(&mut self, draw_context: &mut OsdDrawContext<'_, D>) -> bool {
        if self.active_element_index >= self.active_element_count {
            self.active_element_index = 0;
            return false;
        }

        let element_id = self.active_elements[self.active_element_index];

        if !self.background_layer_supported && !self.background_rendered {
            //  && DrawBackgroundFunctions[element]
            // If the background layer isn't supported then we
            // have to draw the element's static layer as well.
            self.background_rendered = self.draw_element_background_by_id(element_id, draw_context).await;
            // After the background always come back to check for foreground
            return true;
        }

        if self.draw_element_by_id(element_id, draw_context).await {
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
                &self.active_element.buf.buf,
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
                &self.active_element.buf.buf,
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

    pub async fn draw_element_by_id<D: Display>(
        &mut self,
        element_id: OsdElementId,
        draw_context: &mut OsdDrawContext<'_, D>,
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
            element_type: Self::element_type(position),
            id: element_id,
            pos_x: Self::pos_x(position),
            pos_y: Self::pos_y(position),
            rendered: true,
            draw_element: true,
            ..Default::default()
        };

        // TODO: need to check drawing of SYS elements
        // Call the element drawing function
        if self.draw_element(draw_context).await {
            self.display_pending_foreground = true;
        }

        self.active_element.rendered
    }

    pub async fn draw_element_background_by_id<D: Display>(
        &mut self,
        element_id: OsdElementId,
        draw_context: &mut OsdDrawContext<'_, D>,
    ) -> bool {
        /*if (!DrawBackgroundFunctions[element_index]) {
            return true;
        }*/
        self.active_element = OsdElement {
            element_type: Self::element_type(self.config.positions[element_id as usize]),
            id: element_id,
            pos_x: Self::pos_x(self.config.positions[element_id as usize]),
            pos_y: Self::pos_y(self.config.positions[element_id as usize]),
            rendered: true,
            draw_element: true,
            ..Default::default()
        };

        if self.draw_element_background(draw_context).await {
            self.display_pending_background = true;
        }

        self.active_element.rendered
    }

    // TODO: we need to clear the screen (async) before calling this.
    pub async fn draw_active_elements_background<D: Display>(&mut self, draw_context: &mut OsdDrawContext<'_, D>) {
        if self.background_layer_supported {
            draw_context.display_port.layer_select(DisplayPortLayer::Background);
            //draw_context.display_port.clear_screen();
            for element_id in self.active_elements {
                while !self.draw_element_background_by_id(element_id, draw_context).await {}
            }
            draw_context.display_port.layer_select(DisplayPortLayer::Foreground);
        }
    }

    pub fn add_active_elements(&mut self, sensors: SensorFlags) {
        for element in OSD_ELEMENT_DISPLAY_ORDER {
            self.add_active_element(*element);
        }
        #[cfg(feature = "gps")]
        if sensors.is_set(SensorFlags::GPS) {
            self.add_active_element(OsdElementId::GpsSats);
            self.add_active_element(OsdElementId::GpsSpeed);
            self.add_active_element(OsdElementId::GpsLat);
            self.add_active_element(OsdElementId::GpsLon);
            self.add_active_element(OsdElementId::HomeDistance);
            self.add_active_element(OsdElementId::HomeDirection);
            self.add_active_element(OsdElementId::FlightDistance);
            self.add_active_element(OsdElementId::Efficiency);
        }
    }

    #[allow(unused)]
    pub fn analyze_active_elements<D: Display>(&mut self, sensors: SensorFlags, draw_context: &mut OsdDrawContext<D>) {
        self.add_active_elements(sensors);
        self.draw_active_elements_background(draw_context);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn update_attitude(&mut self, roll_angle_degrees: f32, pitch_angle_degrees: f32, yaw_angle_degrees: f32) {
        self.roll_angle_degrees = (roll_angle_degrees + 0.5).floor() as i32;
        self.pitch_angle_degrees = (pitch_angle_degrees + 0.5).floor() as i32;
        self.yaw_angle_degrees = (yaw_angle_degrees + 0.5).floor() as i32;
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
