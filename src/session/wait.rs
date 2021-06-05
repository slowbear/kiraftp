use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn wait(&mut self) -> IOResult {
        self.control_stream.write(b"200 NOOP ok.\r\n").await?;
        Ok(())
    }
}
