// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::FTPSession;
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn wait(&mut self) -> tokio::io::Result<()> {
        self.control_stream.write(b"200 NOOP ok.\r\n").await?;
        Ok(())
    }
}
