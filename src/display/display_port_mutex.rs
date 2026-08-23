use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use static_cell::StaticCell;

static DISPLAY_PORT_MUTEX: StaticCell<super::DisplayPortMutex> = StaticCell::new();

#[cfg(feature = "max7456")]
pub type DisplayPortMax7456 = super::DisplayPortMax7456<DisplaySpi>;
#[cfg(not(feature = "max7456"))]
pub type DisplayPortMax7456 = super::DisplayPortMock;

/*
#[cfg(feature = "max7456")]
pub type DisplayPortMutex = Mutex<CriticalSectionRawMutex, DisplayPortMax7456>;
#[cfg(not(feature = "max7456"))]
pub type DisplayPortMutex = Mutex<CriticalSectionRawMutex, super::DisplayPortMock>;
*/

pub type DisplayPortMutex = Mutex<CriticalSectionRawMutex, DisplayPortMax7456>;

pub fn display_port_mutex_init() -> &'static mut DisplayPortMutex {
    #[rustfmt::skip]
        let display_port = {
            #[cfg(feature = "max7456")] { super::DisplayPortMax7456::new(aux_pio_spi) }
            #[cfg(not(feature = "max7456"))] { super::DisplayPortMock::default() }
        };
    DISPLAY_PORT_MUTEX.init(embassy_sync::mutex::Mutex::new(display_port))
}
