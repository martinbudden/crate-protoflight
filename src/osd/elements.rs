#![cfg(feature = "osd")]
#![allow(unused)]

use crate::osd::OsdElementsConfig;

#[derive(Clone, Copy, Debug, PartialEq)]
enum OsdEnum {
    RssiValue,
    MainBatteryVoltage,
    Crosshairs,
}

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
    pub index: u8,
    pub pos_x: u8,
    pub pos_y: u8,
    pub offset_x: u8,
    pub offset_y: u8,
    pub attr: u8,
    pub rendered: u8,
    pub draw_element: u8,
}

impl OsdElement {
    const BUFFER_LENGTH: usize = 32;
}
impl OsdElement {
    pub const fn new() -> Self {
        Self {
            buf: [0u8; Self::BUFFER_LENGTH],
            element_type: OsdElementType::Type1,
            index: 0,
            pos_x: 0,
            pos_y: 0,
            offset_x: 0,
            offset_y: 0,
            attr: 0,
            rendered: 0,
            draw_element: 0,
        }
    }
}

impl Default for OsdElement {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OsdElements {
    config: OsdElementsConfig,
    active_element: OsdElement,
    active_elements: [OsdEnum; Self::COUNT],
    active_element_count: usize,
    active_element_index: usize,
    profile: u16,
}

#[rustfmt::skip]
impl OsdElements {
    pub const COUNT: usize = 32;
    pub const ELEMENT_BITS_POS: u16 = 14;
    pub const PROFILE_BITS_POS: u16 = 12;
    pub const XY_POSITION_BITS: u16 = 6; // 6 bits gives a range 0-63
    pub const ELEMENT_TYPE_MASK: u16 = 0b_1100_0000_0000_0000; // bits 14-15
    pub const PROFILE_MASK:      u16 = 0b_0011_0000_0000_0000;
    pub const Y_POSITION_MASK:   u16 = 0b_0000_1111_1100_0000;
    pub const X_POSITION_MASK:   u16 = 0b_0000_0000_0011_1111;
}

impl OsdElements {
    pub fn element_type(x: u16) -> OsdElementType {
        OsdElementType::from((x & Self::ELEMENT_TYPE_MASK) >> Self::ELEMENT_BITS_POS)
    }
    pub fn profile_flag(x: u16) -> u16 {
        1 << (x - 1 + Self::PROFILE_BITS_POS)
    }
    pub fn element_visible(value: u16, profile: u16) -> bool {
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
    fn config(&self) -> OsdElementsConfig {
        self.config
    }
    fn set_config(&mut self, config: OsdElementsConfig) {
        self.config = config;
    }
    fn add_active_element(&mut self, element: OsdEnum) {
        if Self::element_visible(self.config.positions[element as usize], self.profile) {
            self.active_elements[self.active_element_count] = element;
            self.active_element_count += 1;
        }
    }
}
