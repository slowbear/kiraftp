// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::FTPSession;
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn quit(&mut self) -> tokio::io::Result<()> {
        self.control_stream.write(b"221 Goodbye.\r\n").await?;
        Ok(())
    }
}
