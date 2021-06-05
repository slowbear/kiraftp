use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    // 只支持F(File)结构，与主流FTP服务器一至
    pub async fn set_file_struct(&mut self, stru: &str) -> IOResult {
        self.control_stream
            .write(if stru == "F" {
                b"200 Structure set to F.\r\n"
            } else {
                b"504 Bad STRU command.\r\n"
            })
            .await?;
        Ok(())
    }
}
