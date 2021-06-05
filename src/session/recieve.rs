use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn recieve(&mut self, path: &str) -> IOResult {
        // TODO: 接收文件
        Ok(())
    }
}
