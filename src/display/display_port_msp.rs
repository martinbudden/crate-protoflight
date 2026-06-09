use core::ops::Deref;

use crate::display::{Display, DisplayClear, DisplayPort, DisplayPortDeviceType, DisplayPortLayer};

struct Commands {}

#[allow(unused)]
impl Commands {
    // MSP Display Port commands
    const HEARTBEAT: u8 = 0;
    /// Release the display after clearing and updating.
    const RELEASE: u8 = 1;
    /// Clear the display.
    const CLEAR_SCREEN: u8 = 2;
    /// Write a string at given coordinates.
    const WRITE_STRING: u8 = 3;
    /// Trigger a screen draw.
    const DRAW_SCREEN: u8 = 4;
    /// Not used. Reserved by Ardupilot and INAV.
    const OPTIONS: u8 = 5;
    // Display system element displayportSystemElement_e at given coordinates.
    const SYS: u8 = 6;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayPortMsp {
    pub display_port: DisplayPort,
}

#[allow(unused)]
impl DisplayPortMsp {
    const MSP_DISPLAYPORT: u16 = 182; // out message: External OSD displayport mode

    const ATTR_VERSION: u8 = 0b_1000_0000; // BIT(7) Format indicator; must be zero for V2 and V1
    const ATTR_BLINK: u8 = 0b_0100_0000; // BIT(6) Device local blink
    const ATTR_FONT: u8 = 0b_0000_0011; // (BIT(0) | BIT(1)) Select bank of 256 characters as per severity
    const ATTR_MASK: u8 = Self::ATTR_VERSION | Self::ATTR_BLINK | Self::ATTR_FONT;
}

#[allow(unused)]
impl DisplayPortMsp {
    pub const fn new() -> Self {
        Self { display_port: DisplayPort::new(DisplayPortDeviceType::Msp) }
    }
}

impl Deref for DisplayPortMsp {
    type Target = DisplayPort;

    fn deref(&self) -> &Self::Target {
        &self.display_port
    }
}

#[allow(unused)]
impl DisplayPortMsp {
    #[allow(clippy::unused_self)]
    pub fn output_byte(&mut self, _byte: u8) -> usize {
        0
    }

    #[allow(clippy::unused_self)]
    pub fn output_slice(&mut self, data: &[u8]) -> usize {
        //let len = data.len();
        0
    }

    pub fn write_string(&mut self, x: u8, y: u8, text: &[u8], attr: u8) -> usize {
        const MSP_OSD_MAX_STRING_LENGTH: usize = 30;
        let mut buf = [0u8; MSP_OSD_MAX_STRING_LENGTH + 4];

        buf[0] = Commands::WRITE_STRING;
        buf[1] = x;
        buf[2] = y;

        let mut attr_byte = 0;
        if (attr & DisplayPort::BLINK) != 0 {
            attr_byte |= Self::ATTR_BLINK;
        }
        buf[3] = attr_byte;

        let len = text.len().min(MSP_OSD_MAX_STRING_LENGTH);
        buf[4..4 + len].copy_from_slice(&text[..len]);

        // Pass ONLY the exact window of bytes you want to transmit
        self.output_slice(&buf[..4 + len])
    }
}

impl Display for DisplayPortMsp {
    fn display_port(&self) -> &DisplayPort {
        &self.display_port
    }

    fn clear_screen(&mut self, _display_clear: DisplayClear) {
        _ = self.output_byte(Commands::CLEAR_SCREEN);
    }

    async fn draw_screen(&mut self) -> Result<bool, &'static str> {
        _ = self.output_byte(Commands::DRAW_SCREEN);
        Ok(false)
    }

    fn redraw(&self) {}
    #[allow(clippy::cast_possible_truncation)]
    fn heartbeat(&mut self) -> i32 {
        self.output_byte(Commands::HEARTBEAT).cast_signed() as i32
    }

    fn write_char(&mut self, x: u8, y: u8, c: u8, attr: u8) -> usize {
        let s = [c];
        self.write_string(x, y, &s, attr)
    }

    fn write_string(&mut self, x: u8, y: u8, text: &[u8], attr: u8) -> usize {
        Self::write_string(self, x, y, text, attr)
    }
    fn layer_supported(&self, _layer: DisplayPortLayer) -> bool {
        true
    }
    fn layer_select(&mut self, layer: DisplayPortLayer) {
        self.display_port.set_active_layer(layer);
    }
    fn layer_copy(&mut self, _src: DisplayPortLayer, _dst: DisplayPortLayer) {}

    fn begin_transaction(&self, _options: u8) {}
    fn commit_transaction(&self) {}
    fn is_transfer_in_progress(&self) -> bool {
        false
    }
    fn check_ready(&self, _val: bool) -> bool {
        true
    }
}
