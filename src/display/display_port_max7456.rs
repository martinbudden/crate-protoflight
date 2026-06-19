//#![cfg(feature = "max7456")]

use embedded_hal_async::spi::SpiBus;

use crate::display::{Display, DisplayPort, DisplayPortDeviceType, DisplayPortLayer, DisplayPortLayers};
use core::ops::Deref;

const SPI_BUFFER_SIZE: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayPortMax7456<SPI> {
    display_layers: DisplayPortLayers,
    shadow_buffer: [u8; DisplayPort::VIDEO_BUFFER_PAL_CHARACTER_COUNT],
    buffer: [u8; 32],
    spi_buffer: [u8; SPI_BUFFER_SIZE], // DMA buffer

    pub font_is_loading: bool,
    pub display_memory_mode_reg: u8,
    //    pub bus: SpiBus,                        // Your low-level SPI/DMA interface
    spi_device: SPI, // Wrapped generic Embassy SPI peripheral
}

#[allow(unused)]
impl<SPI: SpiBus> DisplayPortMax7456<SPI> {
    const VIDEO_BUFFER_DISABLE: u8 = 0x01;
    const MAX7456_RESET: u8 = 0x02;
    const VERTICAL_SYNC_NEXT_VSYNC: u8 = 0x04;
    const OSD_ENABLE: u8 = 0x08;
    // 10 MHz max SPI frequency
    const MAX_SPI_CLOCK_FREQUENCY_HZ: u32 = 10_000_000;
    const INITIAL_SPI_CLOCK_FREQUENCY_HZ: u32 = 5_000_000;
    // MAX7456 Register Addresses & Constants
    const MAX_7456ADD_DMM: u8 = 0x04;
    const MAX_7456ADD_DMAH: u8 = 0x05;
    const MAX_7456ADD_DMAL: u8 = 0x06;
    const MAX_7456ADD_DMDI: u8 = 0x07;

    const DMM_AUTO_INC: u8 = 0x02;
    const END_STRING: u8 = 0xFF;
}

#[allow(unused)]
impl<SPI: SpiBus> DisplayPortMax7456<SPI> {
    /// Create a new MAX7456 OSD driver instance wrapping an Embassy SPI peripheral.
    pub fn new(spi_device: SPI) -> Self {
        Self {
            display_layers: DisplayPortLayers::new(DisplayPortDeviceType::Max7456),
            shadow_buffer: [0u8; DisplayPort::VIDEO_BUFFER_PAL_CHARACTER_COUNT],
            buffer: [0u8; 32],
            spi_buffer: [0u8; SPI_BUFFER_SIZE],
            font_is_loading: false,
            display_memory_mode_reg: 0,
            // Capture the runtime hardware peripheral instance passed into the constructor
            spi_device,
        }
    }
}

impl<SPI: SpiBus> Deref for DisplayPortMax7456<SPI> {
    type Target = DisplayPort;

    fn deref(&self) -> &Self::Target {
        &self.display_layers.display_port
    }
}

#[allow(unused)]
impl<SPI: SpiBus> DisplayPortMax7456<SPI> {
    /// When clearing the shadow buffer we fill with 0 so that the characters will
    /// be flagged as changed when compared to the 0x20 used in the layer buffers.
    pub fn clear_shadow_buffer(&mut self) {
        self.shadow_buffer.fill(0);
    }

    pub async fn draw_screen(&mut self) -> Result<bool, SPI::Error> {
        if self.font_is_loading {
            return Ok(false);
        }

        let active_idx = self.display_layers.display_port.active_layer() as usize;
        let Some(layer) = self.display_layers.display_layers.get_mut(active_idx) else {
            return Ok(false);
        };
        let active_buffer = &mut layer.buffer;
        let screen_size = self.display_layers.display_port.screen_size();

        let mut spi_buffer_index = 0;
        let mut set_address = true;
        let mut auto_inc = false;
        let mut data_was_sent = false;

        // We need up to 8 bytes to transmit a character.
        let max_safe_index = SPI_BUFFER_SIZE.saturating_sub(8);

        for pos in 0..screen_size {
            let mut char_to_write = active_buffer[pos];
            let shadow_char = self.shadow_buffer[pos];

            if char_to_write != shadow_char {
                if char_to_write == 0xFF {
                    char_to_write = b' ';
                    active_buffer[pos] = b' ';
                }
                self.shadow_buffer[pos] = char_to_write;

                // Flush the buffer if we don't have enough room to safely add characters.
                if spi_buffer_index >= max_safe_index {
                    Self::flush_dma_buffer(
                        &mut self.spi_device,
                        &mut self.spi_buffer,
                        self.display_memory_mode_reg,
                        spi_buffer_index,
                        auto_inc,
                        set_address,
                    )
                    .await?;
                    spi_buffer_index = 0;
                    auto_inc = false;
                    set_address = true;
                    data_was_sent = true;
                }

                if set_address || !auto_inc {
                    self.spi_buffer[spi_buffer_index] = Self::MAX_7456ADD_DMM;
                    let next_pos = pos + 1;
                    if next_pos < screen_size && active_buffer[next_pos] != self.shadow_buffer[next_pos] {
                        self.spi_buffer[spi_buffer_index + 1] = self.display_memory_mode_reg | Self::DMM_AUTO_INC;
                        auto_inc = true;
                    } else {
                        self.spi_buffer[spi_buffer_index + 1] = self.display_memory_mode_reg;
                        auto_inc = false;
                    }
                    spi_buffer_index += 2;

                    self.spi_buffer[spi_buffer_index] = Self::MAX_7456ADD_DMAH;
                    self.spi_buffer[spi_buffer_index + 1] = pos.to_le_bytes()[1];
                    self.spi_buffer[spi_buffer_index + 2] = Self::MAX_7456ADD_DMAL;
                    self.spi_buffer[spi_buffer_index + 3] = pos.to_le_bytes()[0];
                    spi_buffer_index += 4;
                    set_address = false;
                }

                self.spi_buffer[spi_buffer_index] = Self::MAX_7456ADD_DMDI;
                self.spi_buffer[spi_buffer_index + 1] = char_to_write;
                spi_buffer_index += 2;
            } else if !set_address {
                set_address = true;
                if auto_inc {
                    self.spi_buffer[spi_buffer_index] = Self::MAX_7456ADD_DMDI;
                    self.spi_buffer[spi_buffer_index + 1] = Self::END_STRING;
                    spi_buffer_index += 2;
                }
            }
        }

        if auto_inc {
            if !set_address {
                self.spi_buffer[spi_buffer_index] = Self::MAX_7456ADD_DMDI;
                self.spi_buffer[spi_buffer_index + 1] = Self::END_STRING;
                spi_buffer_index += 2;
            }
            self.spi_buffer[spi_buffer_index] = Self::MAX_7456ADD_DMM;
            self.spi_buffer[spi_buffer_index + 1] = self.display_memory_mode_reg;
            spi_buffer_index += 2;
        }

        if spi_buffer_index > 0 {
            let tx_slice = &self.spi_buffer[..spi_buffer_index];
            self.spi_device.write(tx_slice).await?;
            data_was_sent = true;
        }

        Ok(data_was_sent)
    }

    /// Associated function (no `self`) that flushes dma buffer..
    async fn flush_dma_buffer(
        spi_device: &mut SPI,
        spi_buffer: &mut [u8; SPI_BUFFER_SIZE],
        display_memory_mode_reg: u8,
        spi_buffer_index: usize,
        auto_inc: bool,
        set_address: bool,
    ) -> Result<(), SPI::Error> {
        let mut spi_buf_idx = spi_buffer_index;
        if auto_inc {
            if !set_address {
                spi_buffer[spi_buf_idx] = Self::MAX_7456ADD_DMDI;
                spi_buffer[spi_buf_idx + 1] = Self::END_STRING;
                spi_buf_idx += 2;
            }
            spi_buffer[spi_buf_idx] = Self::MAX_7456ADD_DMM;
            spi_buffer[spi_buf_idx + 1] = display_memory_mode_reg;
            spi_buf_idx += 2;
        }

        let tx_slice = &spi_buffer[..spi_buf_idx];
        spi_device.write(tx_slice).await?;

        Ok(())
    }
}

impl<SPI: SpiBus> Display for DisplayPortMax7456<SPI> {
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
        // Disambiguate by calling the underlying struct method explicitly and awaiting it
        // We map the SPI error to a trait-compatible string error slice
        //Self::draw_screen(self).await.map_err(|_| "SPI Hardware Transfer Failed")
        match Self::draw_screen(self).await {
            Ok(transferring) => Ok(transferring),
            Err(_hardware_error) => Err("RP2350 SPI Bus hardware error occurred"),
        }
    }
    fn redraw(&self) {}
}
