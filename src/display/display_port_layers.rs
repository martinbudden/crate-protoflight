use crate::display::{DisplayPort, DisplayPortDeviceType, DisplayPortLayer, DisplayPortLayerBuffer};
use core::convert::Infallible;
use core::ops::Deref;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayPortLayers {
    pub display_port: DisplayPort,
    pub display_layers: [DisplayPortLayerBuffer; DisplayPort::LAYER_COUNT],
}

impl DisplayPortLayers {
    pub const fn new(device_type: DisplayPortDeviceType) -> Self {
        Self {
            display_port: DisplayPort::new(device_type),
            display_layers: [DisplayPortLayerBuffer::new(); DisplayPort::LAYER_COUNT],
        }
    }
}

impl Default for DisplayPortLayers {
    fn default() -> Self {
        Self::new(super::DisplayPortDeviceType::Auto)
    }
}

impl Deref for DisplayPortLayers {
    type Target = DisplayPort;

    fn deref(&self) -> &Self::Target {
        &self.display_port
    }
}

impl DisplayPortLayers {
    /// Buffer is filled with the space character (0x20).
    pub fn clear_layer(&mut self, layer: DisplayPortLayer) {
        self.display_layers[layer as usize].buffer.fill(0x20);
    }

    pub fn write_char(&mut self, x: u8, y: u8, c: u8, _attr: u8) -> usize {
        // Validate bounds against the runtime configuration
        if x >= self.display_port.column_count() || y >= self.display_port.row_count() {
            return 0;
        }

        // Fetch the active layer safely
        let active_idx = self.display_port.active_layer() as usize;
        if let Some(layer) = self.display_layers.get_mut(active_idx) {
            // Delegate coordinate translation down to the layer itself
            if let Some(cell) = layer.get_mut(x, y, self.display_port.column_count()) {
                *cell = c;
                return 1;
            }
        }
        0
    }

    pub fn write_string(&mut self, x: u8, y: u8, s: &[u8], _attr: u8) -> usize {
        let column_count = self.display_port.column_count();
        let row_count = self.display_port.row_count();

        if x >= column_count || y >= row_count || s.is_empty() {
            return 0;
        }

        let active_idx = self.display_port.active_layer() as usize;
        let Some(layer) = self.display_layers.get_mut(active_idx) else {
            return 0;
        };

        // Calculate starting index and maximum text space remaining on this line
        let start_idx = (y as usize * column_count as usize) + x as usize;
        let max_line_len = (column_count - x) as usize;

        // Truncate the input string length if it would overflow past the screen edge
        let write_len = s.len().min(max_line_len);
        let bytes_to_write = &s[..write_len];

        // Safely slice out the relevant target sub-array segment.
        // This ensures no out-of-bounds panics can occur inside the loop.
        if let Some(target_window) = layer.buffer.get_mut(start_idx..(start_idx + write_len)) {
            // Zip the target memory spaces with your input characters and overwrite them
            for (cell, &c) in target_window.iter_mut().zip(bytes_to_write.iter()) {
                *cell = c;
            }
            write_len
        } else {
            0
        }
    }

    pub fn layer_copy(&mut self, src: DisplayPortLayer, dst: DisplayPortLayer) {
        // If source and destination are the same, doing work is unnecessary
        if src == dst {
            return;
        }

        let src_idx = src as usize;
        let dst_idx = dst as usize;

        // Ensure indices don't overflow our known fixed array limits
        if src_idx >= DisplayPort::LAYER_COUNT || dst_idx >= DisplayPort::LAYER_COUNT {
            return;
        }

        // Rust safety rule: We cannot borrow from `self.display_layers` twice directly.
        // Instead, we split the slice or manipulate via standard pointer copies safely.
        if src_idx < dst_idx {
            let (src, dst) = self.display_layers.split_at_mut(dst_idx);
            let src_layer = &src[src_idx];
            let dst_layer = &mut dst[0];
            dst_layer.buffer.copy_from_slice(&src_layer.buffer);
        } else {
            let (dst, src) = self.display_layers.split_at_mut(src_idx);
            let src_layer = &src[0];
            let dst_layer = &mut dst[dst_idx];
            dst_layer.buffer.copy_from_slice(&src_layer.buffer);
        }
    }

    /// Asynchronously simulates a screen update by doing nothing.
    /// Returns `Ok(false)` immediately to signal that no data transmission is ongoing.
    #[allow(unused)]
    #[allow(clippy::unused_async)]
    pub async fn draw_screen(&mut self) -> Result<bool, Infallible> {
        Ok(false)
    }
}
