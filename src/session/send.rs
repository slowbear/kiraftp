use tokio::io::AsyncWriteExt;

use super::{FTPSession, IOResult};

impl FTPSession {
    pub async fn send(&mut self, path: &str) -> IOResult {
        // TODO: 发送文件
        if !self.is_loggined {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        Ok(())
    }
}
