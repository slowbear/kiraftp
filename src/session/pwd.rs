// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn print_working_directory(&mut self) -> IOResult {
        if !self.is_loggined {
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
