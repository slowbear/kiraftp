use super::{FTPSession, IOResult};
use crate::utils::fs::{combine, is_dir};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn change_working_directory(&mut self, path: &str) -> IOResult {
        if !self.is_loggined {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        match combine(&self.virtual_root, &self.current_path, path) {
            Some(path) => {
                if is_dir(&path).await {
                    // TODO: 完整的virtual root支持
                    self.current_path = PathBuf::from(
                        path.strip_prefix(&self.virtual_root)
                            .unwrap_or(&self.virtual_root),
                    );
                    self.control_stream
                        .write(b"250 Directory successfully changed.\r\n")
                        .await?;
                } else {
                    self.control_stream
                        .write(b"550 Failed to change directory.\r\n")
                        .await?;
                }
            }
            None => {
                self.control_stream
                    .write(b"550 Failed to change directory.\r\n")
                    .await?;
            }
        }
        Ok(())
    }
}
