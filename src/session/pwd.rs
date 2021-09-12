// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::FTPSession;
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn print_working_directory(&mut self) -> tokio::io::Result<()> {
        if !self.is_logged_in {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        self.control_stream
            .write(
                format!(
                    "257 \"{}\" is the current directory.\r\n",
                    self.current_path.to_string_lossy()
                )
                .as_bytes(),
            )
            .await?;
        Ok(())
    }
}
