#![allow(unused)]
#[cfg(feature = "rtt-debug")]
use rtt_target::{UpChannel, rprint, rprintln};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
pub struct DisplayPortLayerBuffer {
    pub(crate) buffer: [u8; DisplayPort::VIDEO_BUFFER_PAL_CHARACTER_COUNT],
}

impl DisplayPortLayerBuffer {
    pub const ROW_COUNT: usize = DisplayPort::VIDEO_LINES_PAL as usize;
    pub const COLUMN_COUNT: usize = DisplayPort::VIDEO_COLUMNS_SD as usize;

    pub const fn new() -> Self {
        Self { buffer: [0u8; Self::ROW_COUNT * Self::COLUMN_COUNT] }
    }
}

impl Default for DisplayPortLayerBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayPortLayerBuffer {
    /// Safe 2D coordinate lookup that translates (x, y) to a 1D array reference.
    pub fn get_mut(&mut self, x: u8, y: u8, stride: u8) -> Option<&mut u8> {
        let index = (y as usize * stride as usize) + x as usize;
        self.buffer.get_mut(index)
    }
    /// Clear screen and home cursor for rtt.
    #[cfg(feature = "rtt-debug")]
    pub fn rtt_init() {
        rprint!("\x1b[2J\x1b[H");
    }

    /// Dump screen for rtt.
    #[cfg(feature = "rtt-debug")]
    pub fn rtt_dump(&self, overwrite: bool) {
        if overwrite {
            // Move cursor up 16 lines for overwrite
            rprint!("\x1b[16A");
        }
        // Render the screen row by row
        for row in 0..Self::ROW_COUNT {
            let start = row * Self::COLUMN_COUNT;
            let end = start + Self::COLUMN_COUNT;
            let line_bytes = &self.buffer[start..end];

            // Convert slice safely to string or fallback to characters
            if let Ok(line_str) = core::str::from_utf8(line_bytes) {
                rprintln!("{}", line_str);
            } else {
                for &byte in line_bytes {
                    rprint!("{}", byte as char);
                }
                rprintln!();
            }
        }
    }

    #[cfg(feature = "rtt-debug")]
    pub fn rtt_dump_fast(&self, channel: &mut UpChannel, overwrite: bool) {
        /*
            NOTE: to use this we need to set up channels at startup, eg:
            let channels = rtt_target::rtt_init! {
                up: {
                    0: {
                        size: 2048,
                        mode: NoBlockSkip,
                        name: "Terminal"
                    }
                }
            };

            let mut rtt_display_channel = channels.up.0;
        */
        if overwrite {
            // Move cursor up 16 lines for overwrite
            _ = channel.write(b"\x1b[16A");
        }
        // Render the screen row by row
        for row in 0..Self::ROW_COUNT {
            let start = row * Self::COLUMN_COUNT;
            let end = start + Self::COLUMN_COUNT;
            let line_bytes = &self.buffer[start..end];

            // Write the raw display data chunk directly to RAM
            _ = channel.write(line_bytes);

            // Append terminal line ending
            _ = channel.write(b"\n");
        }
    }
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
    /// blink attribute bit.
    pub const BLINK: u8 = 0x80;

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
    pub const fn new(device_type: DisplayPortDeviceType) -> Self {
        Self {
            device_type,
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
        Self::new(DisplayPortDeviceType::Auto)
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

    fn display_port_mut(&mut self) -> &mut DisplayPort;

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

    fn heartbeat(&mut self) -> i32;

    fn write_string(&mut self, x: u8, y: u8, text: &[u8], attr: u8) -> usize;
    fn write_char(&mut self, x: u8, y: u8, c: u8, attr: u8) -> usize;

    fn layer_supported(&self, layer: DisplayPortLayer) -> bool;
    fn layer_select(&mut self, layer: DisplayPortLayer);
    fn layer_copy(&mut self, src: DisplayPortLayer, dst: DisplayPortLayer);

    fn begin_transaction(&mut self, option: u8);
    fn commit_transaction(&mut self);
    fn is_transfer_in_progress(&self) -> bool;
    fn check_ready(&self, val: bool) -> bool;

    async fn clear_screen(&mut self);
    async fn draw_screen(&mut self) -> Result<bool, &'static str>;
    fn redraw(&self);
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
