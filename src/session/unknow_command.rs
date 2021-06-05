use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn unknow_command(&mut self) -> IOResult {
        self.control_stream
            .write(b"500 Unknow command.\r\n")
            .await?;
        Ok(())
    }
}
