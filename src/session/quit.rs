use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn quit(&mut self) -> IOResult {
        self.control_stream.write(b"221 Goodbye.\r\n").await?;
        Ok(())
    }
}
