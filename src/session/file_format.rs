// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::FTPSession;
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn set_transfer_mode(&mut self, mode: &str) -> tokio::io::Result<()> {
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
