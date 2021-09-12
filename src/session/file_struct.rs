// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::FTPSession;
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn set_file_struct(&mut self, stru: &str) -> tokio::io::Result<()> {
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
