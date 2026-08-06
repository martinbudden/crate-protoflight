#![cfg(feature = "osd")]

use crate::{
    display::{Display, DisplayPortLayer, DisplayPortSeverity},
    osd::{
        OsdElementsConfig,
        display::OsdDrawContext,
        elements_draw::{OSD_ELEMENT_DISPLAY_ORDER, OsdElementId},
        fixed_buf::FixedBuf,
        //osd_buffer_cursor::OsdBufferCursor,
    },
    sensors::SensorFlags,
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

/// Retains state of elements that require multiple steps to draw them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OsdElementState {
    pub horizon_x: i32,
    pub stick_overlay_render_phase: OsdStickOverlayRenderPhase,
    pub stick_overlay_y: u8,
    pub sidebar_y: i8,
    pub sidebar_render_level: bool,
    pub camera_frame_render_phase: OsdStickCameraFrameRenderPhase,
    pub camera_frame_i: u8,
}

impl Default for OsdElementState {
    fn default() -> Self {
        Self::new()
    }
}

impl OsdElementState {
    pub const fn new() -> Self {
        Self {
            horizon_x: -4,
            stick_overlay_render_phase: OsdStickOverlayRenderPhase::Vertical,
            stick_overlay_y: 0,
            sidebar_y: 0,
            sidebar_render_level: false,
            camera_frame_render_phase: OsdStickCameraFrameRenderPhase::Top,
            camera_frame_i: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OsdElement {
    pub fixed_buf: FixedBuf<{ OsdElement::BUFFER_LENGTH }>,
    pub element_type: OsdElementType,
    pub id: OsdElementId,
    pub pos_x: u8,
    pub pos_y: u8,
    pub offset_x: u8,
    pub offset_y: u8,
    pub attr: DisplayPortSeverity,
    pub rendered: bool,
    pub draw_element: bool,
    pub osd_cap_alarm: i16,
    pub state: OsdElementState,
    // Cache mapping historical rendering durations per active element index.
    // pub duration_fraction_us: u32,
}

impl Default for OsdElement {
    fn default() -> Self {
        Self::new()
    }
}

impl OsdElement {
    pub const BUFFER_LENGTH: usize = 32;

    pub const fn new() -> Self {
        Self {
            fixed_buf: FixedBuf::new(),
            element_type: OsdElementType::Type1,
            id: OsdElementId::Altitude,
            pos_x: 0,
            pos_y: 0,
            offset_x: 0,
            offset_y: 0,
            attr: DisplayPortSeverity::Normal,
            rendered: false,
            draw_element: false,
            osd_cap_alarm: 0,
            state: OsdElementState::new(),
            // duration_fraction_us: 0,
        }
    }
}

impl OsdElement {
    /// Overwrites the buffer completely with a static string and fills the rest with 0.
    pub fn write_string(&mut self, string: &str) {
        let bytes = string.as_bytes();
        self.write_slice(bytes);
    }

    pub fn write_slice(&mut self, slice: &[u8]) {
        let len = slice.len().min(Self::BUFFER_LENGTH);

        self.fixed_buf.bytes[..len].copy_from_slice(&slice[..len]);
        self.fixed_buf.bytes[len..].fill(0);
    }

    /*/// Flexible multi-part writer that allows concatenating text and numbers manually.
    /// Returns the number of bytes written.
    pub fn write_custom<F>(&mut self, write_logic: F) -> usize
    where
        F: FnOnce(&mut OsdBufferCursor),
    {
        self.fixed_buf.bytes.fill(0);

        let mut cursor = OsdBufferCursor { buf: &mut self.fixed_buf.bytes, pos: 0 };

        write_logic(&mut cursor);

        cursor.pos
    }*/
}

#[allow(clippy::struct_field_names)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OsdElementsCache {
    pub roll_angle_degrees: i32,
    pub pitch_angle_degrees: i32,
    pub yaw_angle_degrees: i32,
}

impl Default for OsdElementsCache {
    fn default() -> Self {
        Self::new()
    }
}

impl OsdElementsCache {
    pub const fn new() -> Self {
        Self { roll_angle_degrees: 0, pitch_angle_degrees: 0, yaw_angle_degrees: 0 }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OsdElements {
    positions: [u16; OsdElements::COUNT],
    current_element: OsdElement,
    active_elements: [OsdElementId; Self::COUNT],
    active_element_count: usize,
    profile: u8,
    background_rendered: bool,
    display_pending_foreground: bool,
    display_pending_background: bool,
    display_supports_background_layer: bool,
    cache: OsdElementsCache,
}

impl OsdElements {
    pub const fn new(display_supports_background_layer: bool) -> Self {
        Self {
            positions: [0u16; OsdElements::COUNT],
            current_element: OsdElement::new(),
            active_elements: [OsdElementId::Rssi; Self::COUNT],
            active_element_count: 0,
            profile: 0,
            background_rendered: false,
            display_pending_foreground: false,
            display_pending_background: false,
            display_supports_background_layer,
            cache: OsdElementsCache::new(),
        }
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

#[allow(clippy::unused_self)]
impl OsdElements {
    /// Simulates a system uptime lookup tool in microseconds.
    #[allow(unused)]
    fn time_us() -> u32 {
        0
    }

    pub fn element_type(x: u16) -> OsdElementType {
        OsdElementType::from((x & Self::ELEMENT_TYPE_MASK) >> Self::ELEMENT_BITS_POS)
    }

    #[allow(unused)]
    pub fn profile_flag(x: u16) -> u16 {
        1 << (x - 1 + Self::PROFILE_BITS_POS)
    }

    #[allow(unused)]
    pub fn set_profile(&mut self, profile: u8) {
        self.profile = profile.clamp(0, 1);
    }

    pub fn element_visible(value: u16, profile: u8) -> bool {
        ((value & Self::PROFILE_MASK) >> Self::PROFILE_BITS_POS) & (1 << profile) == 0
    }

    pub fn pos_x(x: u16) -> u8 {
        (x & Self::X_POSITION_MASK) as u8
    }

    #[allow(unused)]
    pub fn pos_y(x: u16) -> u8 {
        ((x >> Self::XY_POSITION_BITS) & Self::X_POSITION_MASK) as u8
    }

    #[allow(unused)]
    pub fn pos(x: u16, y: u16) -> u16 {
        (x & Self::X_POSITION_MASK) | ((y & Self::X_POSITION_MASK) << Self::XY_POSITION_BITS)
    }

    #[allow(unused)]
    pub fn set_positions(&mut self, config: OsdElementsConfig) {
        self.positions = config.positions;
    }

    pub fn add_active_element(&mut self, element: OsdElementId) {
        if Self::element_visible(self.positions[element as usize], self.profile) {
            self.active_elements[self.active_element_count] = element;
            self.active_element_count += 1;
        }
    }

    pub fn set_current_element_by_index(&mut self, element_index: usize) -> bool {
        if element_index > self.active_element_count || element_index >= self.positions.len() {
            return false;
        }
        let element_id = self.active_elements[element_index];
        self.set_current_element(element_id);
        true
    }

    pub fn set_current_element(&mut self, element_id: OsdElementId) {
        let position = self.positions[element_id as usize];
        self.current_element = OsdElement {
            element_type: Self::element_type(position),
            id: element_id,
            pos_x: Self::pos_x(position),
            pos_y: Self::pos_y(position),
            rendered: true,
            draw_element: true,
            ..Default::default()
        };
    }

    pub async fn draw_current_element<D: Display>(&mut self, draw_context: &mut OsdDrawContext<'_, D>) -> bool {
        // const OSD_EXEC_TIME_SHIFT: u32 = 5;

        //let start_element_time = Self::time_us();

        // Draw the background before the foreground.
        if !self.display_supports_background_layer && !self.background_rendered {
            // If the display doesn't support a background layer then we need to draw the element background now.
            self.current_element.rendered = true;
            let (drawn, rendered) = Self::draw_element_background(draw_context, &mut self.current_element).await;
            if drawn {
                self.display_pending_background = true;
            }
            self.background_rendered = rendered;
        }

        // TODO: need to check drawing of SYS elements
        // Call the element drawing function
        self.current_element.rendered = true;
        let (drawn, rendered) =
            Self::draw_element_foreground(draw_context, &mut self.current_element, self.cache).await;
        if drawn {
            self.display_pending_foreground = true;
        }

        /*let execute_time_us = Self::time_us() - start_element_time;

        if execute_time_us > (self.current_element.duration_fraction_us >> OSD_EXEC_TIME_SHIFT) {
            self.current_element.duration_fraction_us = execute_time_us << OSD_EXEC_TIME_SHIFT;
        } else if self.current_element.duration_fraction_us > 0 {
            // Slowly decay the max time
            self.current_element.duration_fraction_us -= 1;
        }*/

        rendered
    }

    pub fn display_current_element<D: Display>(&mut self, display_port: &mut D) -> bool {
        // If there's a previously drawn background string to be displayed, do that
        if self.display_pending_background {
            _ = display_port.write_slice(
                self.current_element.pos_x + self.current_element.offset_x,
                self.current_element.pos_y + self.current_element.offset_y,
                &self.current_element.fixed_buf.bytes,
                self.current_element.attr,
            );
            self.display_pending_background = false;
            return self.display_pending_foreground;
        }
        // If there's a previously drawn foreground string to be displayed, do that
        if self.display_pending_foreground {
            _ = display_port.write_slice(
                self.current_element.pos_x + self.current_element.offset_x,
                self.current_element.pos_y + self.current_element.offset_y,
                &self.current_element.fixed_buf.bytes,
                self.current_element.attr,
            );
            self.display_pending_foreground = false;
        }
        false
    }

    pub fn draw_spec(&self) -> bool {
        true
    }

    // TODO: we need to clear the screen (async) before calling this.
    pub async fn draw_background_for_all_active_elements<D: Display>(
        &mut self,
        draw_context: &mut OsdDrawContext<'_, D>,
    ) {
        if self.display_supports_background_layer {
            // If the display supports a background layer then we can all the background now
            // and we don't need to draw the background for each individual element.
            draw_context.display_port.layer_select(DisplayPortLayer::Background);
            for element_id in self.active_elements {
                self.set_current_element(element_id);

                let mut rendered = false;
                while !rendered {
                    (_, rendered) = Self::draw_element_background(draw_context, &mut self.current_element).await;
                }
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
        self.draw_background_for_all_active_elements(draw_context);
    }

    // Cache values that are used by more than one element, so we only have to calculate them once.
    pub fn update_cache(&mut self, roll_angle_degrees: f32, pitch_angle_degrees: f32, yaw_angle_degrees: f32) {
        #[allow(clippy::cast_possible_truncation)]
        {
            self.cache.roll_angle_degrees = (roll_angle_degrees + 0.5).floor() as i32;
            self.cache.pitch_angle_degrees = (pitch_angle_degrees + 0.5).floor() as i32;
            self.cache.yaw_angle_degrees = (yaw_angle_degrees + 0.5).floor() as i32;
        }
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
        //is_full::<OsdElements>();
        is_full::<OsdElementType>();
    }
}
