#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum DisplayPortDeviceType {
    #[default]
    None,
    Auto,
    Max7456,
    Oled,
    FrskyOsd,
    Msp,
    Crsf,
    Hott,
    Srxl,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum DisplayPortLayer {
    #[default]
    Foreground,
    Background,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum DisplayPortBackground {
    #[default]
    Transparent,
    Black,
    Gray,
    LightGray,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum DisplayPortSeverity {
    #[default]
    Normal,
    Info,
    Warning,
    Critical,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayPort {
    device_type: DisplayPortDeviceType,
    background_type: DisplayPortBackground,
    active_layer: DisplayPortLayer,

    max_screen_size: u16,
    row_count: u8,
    column_count: u8,
    pos_x: u8,
    pos_y: u8,
}

impl DisplayPort {
    const SMALL_ARROW_UP: u8 = b'^';
    const SMALL_ARROW_DOWN: u8 = b'v';
    const VIDEO_COLUMNS_SD: u8 = 30;
    const VIDEO_LINES_NTSC: u8 = 13;
    const VIDEO_LINES_PAL: u8 = 16;
}

impl DisplayPort {
    pub const fn new() -> Self {
        Self {
            device_type: DisplayPortDeviceType::None,
            background_type: DisplayPortBackground::Transparent,
            active_layer: DisplayPortLayer::Foreground,
            max_screen_size: Self::VIDEO_LINES_PAL as u16 * Self::VIDEO_COLUMNS_SD as u16,
            row_count: Self::VIDEO_LINES_PAL,
            column_count: Self::VIDEO_COLUMNS_SD,
            pos_x: 0,
            pos_y: 0,
        }
    }
}

impl Default for DisplayPort {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayPort {
    pub fn device_type(&self) -> DisplayPortDeviceType {
        self.device_type
    }
    pub fn screen_size(&self) -> u32 {
        u32::from(self.row_count) * u32::from(self.column_count)
    }
    pub fn row_count(&self) -> u8 {
        self.row_count
    }
    pub fn column_count(&self) -> u8 {
        self.column_count
    }
}

pub trait Display {
    fn clear_screen(&mut self);
    fn draw_screen(&mut self) -> bool; // Returns true if screen still being transferred
    fn heartbeat(&mut self) -> i32;

    fn write_string(&mut self, x: u8, y: u8, s: &str, attr: u8) -> usize;
    fn write_char(&mut self, x: u8, y: u8, c: u8, attr: u8) -> usize;
}
