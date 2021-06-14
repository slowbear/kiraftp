use std::path::PathBuf;

use super::{FTPSession, IOResult};
use crate::utils::helper::combine;
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn change_working_directory(&mut self, path: &str) -> IOResult {
        if !self.is_loggined {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        // TODO: 未处理不存在路径
        let path = combine(&self.virtual_root, &self.current_path, path).unwrap();
        // TODO: 将is_dir改为异步
        if path.is_dir() {
            self.current_path = PathBuf::from(path.strip_prefix(&self.virtual_root).unwrap());
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
