#![doc = include_str!("../README.md")]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![warn(unused_results)]
#![warn(clippy::pedantic)]
#![warn(clippy::doc_paragraphs_missing_punctuation)]

mod autopilot;
mod barometer_sensors;
mod boards;
mod config;
mod display;
mod drivers;
mod flight;
mod gps;
mod i2c_bus;
mod magnetometer_sensors;
mod multiwii_serial_protocol;
mod optical_flow_sensors;
mod osd;
mod rangefinder_sensors;
mod sensors;
mod tasks;
mod vtx;

// =========================================================================
// MANDATORY EMBEDDED PANIC HANDLER
// =========================================================================

//#[cfg(any(feature = "rp2350", feature = "stm32f405", feature = "stm32", feature = "esp32"))]
#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    crate::tasks::init::init(spawner).await;
}
