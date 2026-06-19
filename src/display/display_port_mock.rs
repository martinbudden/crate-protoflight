use crate::display::{Display, DisplayPort, DisplayPortDeviceType, DisplayPortLayer, DisplayPortLayers};
use core::ops::Deref;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayPortMock {
    display_layers: DisplayPortLayers,
}

impl DisplayPortMock {
    pub const fn new(device_type: DisplayPortDeviceType) -> Self {
        Self { display_layers: DisplayPortLayers::new(device_type) }
    }
}

impl Default for DisplayPortMock {
    fn default() -> Self {
        Self::new(super::DisplayPortDeviceType::Auto)
    }
}

impl Deref for DisplayPortMock {
    type Target = DisplayPort;

    fn deref(&self) -> &Self::Target {
        &self.display_layers.display_port
    }
}

#[allow(unused)]
impl Display for DisplayPortMock {
    fn display_port(&self) -> &DisplayPort {
        &self.display_layers.display_port
    }

    fn display_port_mut(&mut self) -> &mut DisplayPort {
        &mut self.display_layers.display_port
    }

    fn heartbeat(&mut self) -> i32 {
        0
    }

    fn write_char(&mut self, x: u8, y: u8, c: u8, attr: u8) -> usize {
        self.display_layers.write_char(x, y, c, attr)
    }

    fn write_string(&mut self, x: u8, y: u8, s: &[u8], attr: u8) -> usize {
        self.display_layers.write_string(x, y, s, attr)
    }

    fn layer_supported(&self, _layer: DisplayPortLayer) -> bool {
        true
    }

    fn layer_select(&mut self, layer: DisplayPortLayer) {
        self.display_port_mut().set_active_layer(layer);
    }

    fn layer_copy(&mut self, src: DisplayPortLayer, dst: DisplayPortLayer) {
        self.display_layers.layer_copy(src, dst);
    }

    fn begin_transaction(&mut self, option: u8) {
        if option == DisplayPort::DISPLAY_TRANSACTION_OPTION_RESET_DRAWING {
            self.display_layers.clear_layer(DisplayPortLayer::Background);
            self.display_layers.clear_layer(DisplayPortLayer::Foreground);
        }
    }

    fn commit_transaction(&mut self) {}

    fn is_transfer_in_progress(&self) -> bool {
        false
    }

    fn check_ready(&self, _val: bool) -> bool {
        true
    }

    async fn clear_screen(&mut self) {
        self.display_layers.clear_layer(self.active_layer());
    }

    async fn draw_screen(&mut self) -> Result<bool, &'static str> {
        Ok(false)
    }

    fn redraw(&self) {}
}
