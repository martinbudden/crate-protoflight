#![cfg(all(feature = "std", feature = "blackbox"))]

use embassy_futures::yield_now;
use std::{fs::File, io::Write};

#[allow(async_fn_in_trait)]
pub trait SdStorage {
    async fn write_all(&mut self, data: &[u8]) -> Result<(), ()>;
}

pub struct MockSdCard {
    file: File,
}

impl MockSdCard {
    /// # Panics
    #[allow(clippy::expect_used)]
    pub fn new(path: &str) -> Self {
        Self { file: File::create(path).expect("Could not create log file") }
    }
}

impl SdStorage for MockSdCard {
    async fn write_all(&mut self, data: &[u8]) -> Result<(), ()> {
        if data.is_empty() {
            return Ok(());
        }
        self.file.write_all(data).map_err(|_| ())?;
        _ = self.file.flush().ok();
        yield_now().await;
        Ok(())
    }
}
