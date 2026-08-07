use crate::display::{
    Display, DisplayPort, DisplayPortDeviceType, DisplayPortLayer, DisplayPortLayers, DisplayPortSeverity,
};
use core::ops::Deref;

#[derive(Debug, PartialEq)]
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

    fn write_byte(&mut self, x: u8, y: u8, byte: u8, attr: DisplayPortSeverity) -> usize {
        self.display_layers.write_byte(x, y, byte, attr)
    }

    fn write_slice(&mut self, x: u8, y: u8, slice: &[u8], attr: DisplayPortSeverity) -> usize {
        self.display_layers.write_slice(x, y, slice, attr)
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

    async fn transfer_screen(&mut self) -> Result<bool, &'static str> {
        Ok(false)
    }

    fn redraw(&self) {}
}
