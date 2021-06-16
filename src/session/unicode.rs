use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn unicode(&mut self) -> IOResult {
        self.control_stream
            .write(b"200 Always in UTF8 mode.\r\n")
            .await?;
        Ok(())
    }
}