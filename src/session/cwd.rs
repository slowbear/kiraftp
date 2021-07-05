use super::{FTPSession, IOResult};
use crate::utils::fs::is_dir;
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn change_working_directory(&mut self, path: &str) -> IOResult {
        if !self.is_loggined {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        let path = self.current_path.join(path);
        if is_dir(&path).await {
            self.current_path = path.canonicalize().unwrap();
            self.control_stream
                .write(b"250 Directory successfully changed.\r\n")
                .await?;
        } else {
            self.control_stream
                .write(b"550 Failed to change directory.\r\n")
                .await?;
        }
        Ok(())
    }
}
