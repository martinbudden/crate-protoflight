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
mod config;
mod display;
mod drivers;
mod flight;
#[cfg(feature = "gps")]
mod gps;
mod multiwii_serial_protocol;
mod osd;
mod sensors;
mod tasks;
mod vtx;

// =========================================================================
// MANDATORY EMBEDDED PANIC HANDLER
// =========================================================================
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
