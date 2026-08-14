#![cfg(feature = "std")]

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MockUart {
    pub data: &'static [u8],
    pub position: usize,
}

impl embedded_io_async::ErrorType for MockUart {
    type Error = embedded_io_async::ErrorKind;
}

//impl embedded_io_async::Read for MockUart {
impl MockUart {
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        // Check if we have read all mock data
        if self.position >= self.data.len() {
            // Put the testing loop into a brief sleep on EOF to avoid hammering the host CPU
            std::thread::sleep(std::time::Duration::from_millis(10));
            return Ok(0);
        }

        core::future::poll_fn(|_| core::task::Poll::Ready(())).await;
        // Copy matching slice data across
        let remaining = &self.data[self.position..];
        let bytes_to_copy = core::cmp::min(remaining.len(), buf.len());

        buf[..bytes_to_copy].copy_from_slice(&remaining[..bytes_to_copy]);
        self.position += bytes_to_copy;

        Ok(bytes_to_copy)
    }
}
