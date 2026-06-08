#![allow(unused)]
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

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum DisplayClear {
    // Display drivers that can perform screen clearing in the background, e.g. via DMA, should do so.
    // use `displayCheckReady` function to check if the screen clear has been completed.
    #[default]
    None,

    // * when set, the display driver should block until the screen clear has completed, use in synchronous cases
    //   only, e.g. where the screen is cleared and the display is immediately drawn to.
    // * when NOT set, return immediately and do not block unless screen is a simple operation or cannot
    //   be performed in the background.  As with any long delay, waiting can cause task starvation which
    //   can result in RX loss.
    Wait,
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
    pub const SMALL_ARROW_UP: u8 = b'^';
    pub const SMALL_ARROW_DOWN: u8 = b'v';

    pub const LAYER_COUNT: usize = 2;
    pub const VIDEO_COLUMNS_SD: u8 = 30;
    pub const VIDEO_LINES_NTSC: u8 = 13;
    pub const VIDEO_LINES_PAL: u8 = 16;

    pub const VIDEO_BUFFER_NTSC_CHARACTER_COUNT: usize = 390;
    pub const VIDEO_BUFFER_PAL_CHARACTER_COUNT: usize = 480;

    pub const DISPLAY_TRANSACTION_OPTION_NONE: u8 = 0x00;
    pub const DISPLAY_TRANSACTION_OPTION_PROFILED: u8 = 0x01;
    pub const DISPLAY_TRANSACTION_OPTION_RESET_DRAWING: u8 = 0x02;
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
    pub fn background_type(&self) -> DisplayPortBackground {
        self.background_type
    }
    pub fn active_layer(&self) -> DisplayPortLayer {
        self.active_layer
    }
    pub fn set_active_layer(&mut self, layer: DisplayPortLayer) {
        self.active_layer = layer;
    }
    pub fn screen_size(&self) -> usize {
        (u32::from(self.row_count) * u32::from(self.column_count)) as usize
    }
    pub fn row_count(&self) -> u8 {
        self.row_count
    }
    pub fn column_count(&self) -> u8 {
        self.column_count
    }
}

pub trait Display {
    fn display_port(&self) -> &DisplayPort;

    fn device_type(&self) -> DisplayPortDeviceType {
        self.display_port().device_type()
    }

    fn screen_size(&self) -> usize {
        self.display_port().screen_size()
    }

    fn row_count(&self) -> u8 {
        self.display_port().row_count()
    }

    fn column_count(&self) -> u8 {
        self.display_port().column_count()
    }

    fn clear_screen(&mut self, display_clear: DisplayClear);
    async fn draw_screen(&mut self) -> Result<bool, &'static str>;

    fn redraw(&self);
    fn heartbeat(&mut self) -> i32;

    fn write_string(&mut self, x: u8, y: u8, s: &[u8], attr: u8) -> usize;
    fn write_char(&mut self, x: u8, y: u8, c: u8, attr: u8) -> usize;
    fn layer_supported(&self, layer: DisplayPortLayer) -> bool;
    fn layer_select(&mut self, layer: DisplayPortLayer);
    fn layer_copy(&mut self, _src: DisplayPortLayer, _dest: DisplayPortLayer);
    fn begin_transaction(&self, _options: u8);
    fn commit_transaction(&self);
    fn is_transfer_in_progress(&self) -> bool;
    fn check_ready(&self, _val: bool) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<DisplayPort>();
    }
}
