// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::FTPSession;
use crate::utils::fs as utfs;
use std::path::Path;
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn change_working_directory(
        &mut self,
        path: impl AsRef<Path>,
    ) -> tokio::io::Result<()> {
        if !self.is_logged_in {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        let path = self.current_path.join(path);
        if utfs::is_dir(&path).await {
            self.current_path = path.canonicalize()?;
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
