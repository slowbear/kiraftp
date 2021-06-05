use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn welcome(&mut self) -> IOResult {
        self.control_stream.write(b"220 TryFTP v0.1.0\r\n").await?;
        Ok(())
    }
}
