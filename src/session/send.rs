use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn send(&mut self, path: &str) -> IOResult {
        // TODO: 发送文件
        Ok(())
    }
}
