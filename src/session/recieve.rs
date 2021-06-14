use super::{FTPSession, IOResult, Transfer};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn recieve(&mut self, path: &str) -> IOResult {
        // TODO: 接收文件
        if !self.is_loggined {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        match &self.transfer {
            Transfer::Active(remote) => {}
            Transfer::Passive(server) => {}
            Transfer::Disable => {
                self.control_stream
                    .write(b"425 Use PORT or PASV first.\r\n")
                    .await?;
            }
        }
        // 一次性链接
        self.transfer = Transfer::Disable;
        Ok(())
    }
}
