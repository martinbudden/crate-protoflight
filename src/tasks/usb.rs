#![cfg(feature = "usb")]
use static_cell::StaticCell;

use crate::{boards::UsbDeviceDriver, tasks::errors::TaskInitError};

static USB_CTX: StaticCell<UsbContext> = StaticCell::new();

/// Context for Usb task.
pub struct UsbContext {
    pub usb_device: UsbDeviceDriver,
}

pub fn init(usb_device: UsbDeviceDriver) -> Result<&'static mut UsbContext, TaskInitError> {
    let ctx = UsbContext { usb_device };
    Ok(USB_CTX.init(ctx))
}

/// Usb background task responsible for keeping the USB stack alive and polling the bus.
#[embassy_executor::task]
pub async fn run(ctx: &'static mut UsbContext) {
    // If initialization fails, then we just return.
    log::info!("         USB: task started");
    ctx.usb_device.run().await
}
