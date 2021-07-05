use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    // 只支持S(Stream)模式，简化实现
    pub async fn set_tranfer_mode(&mut self, mode: &str) -> IOResult {
        let mode = mode.to_ascii_uppercase();
        self.control_stream
            .write(if mode == "S" {
                b"200 Mode set to S.\r\n"
            } else {
                b"504 Bad MODE command.\r\n"
            })
            .await?;
        Ok(())
    }
}
