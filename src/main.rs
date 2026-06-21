#![doc = include_str!("../README.md")]
#![no_std]
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
mod flight;
#[cfg(feature = "gps")]
mod gps;
mod multiwii_serial_protocol;
mod osd;
mod sensors;
mod tasks;
mod vtx;

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    crate::tasks::init::init(spawner).await;
}
