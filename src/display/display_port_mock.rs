use crate::display::{Display, DisplayClear, DisplayPort, DisplayPortLayer};
use core::convert::Infallible;
use core::ops::Deref;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayPortMock {
    pub display_port: DisplayPort,
}

impl DisplayPortMock {
    pub const fn new() -> Self {
        Self { display_port: DisplayPort::new(super::DisplayPortDeviceType::Auto) }
    }
}

impl Default for DisplayPortMock {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for DisplayPortMock {
    type Target = DisplayPort;

    fn deref(&self) -> &Self::Target {
        &self.display_port
    }
}

impl DisplayPortMock {
    /// Asynchronously simulates a screen update by doing nothing.
    /// Returns `Ok(false)` immediately to signal that no data transmission is ongoing.
    #[allow(clippy::unused_async)]
    pub async fn draw_screen(&mut self) -> Result<bool, Infallible> {
        Ok(false)
    }
}

#[allow(unused)]
impl Display for DisplayPortMock {
    fn display_port(&self) -> &DisplayPort {
        &self.display_port
    }

    fn clear_screen(&mut self, display_clear: DisplayClear) {}

    async fn draw_screen(&mut self) -> Result<bool, &'static str> {
        // Call the underlying mock method
        let _ = Self::draw_screen(self).await;
        Ok(false)
    }

    fn redraw(&self) {}

    fn heartbeat(&mut self) -> i32 {
        0
    }
    fn write_string(&mut self, x: u8, y: u8, s: &[u8], attr: u8) -> usize {
        0
    }
    fn write_char(&mut self, x: u8, y: u8, c: u8, attr: u8) -> usize {
        0
    }
    fn layer_supported(&self, layer: DisplayPortLayer) -> bool {
        if layer == DisplayPortLayer::Foreground {
            // Every device must support the foreground (default) layer
            return true;
        }
        false
    }
    fn layer_select(&mut self, layer: DisplayPortLayer) {}
    fn layer_copy(&mut self, src: DisplayPortLayer, dest: DisplayPortLayer) {}

    fn begin_transaction(&self, _options: u8) {}
    fn commit_transaction(&self) {}
    fn is_transfer_in_progress(&self) -> bool {
        false
    }
    fn check_ready(&self, _val: bool) -> bool {
        true
    }
}
