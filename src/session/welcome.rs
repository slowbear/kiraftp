// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::FTPSession;
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn welcome(&mut self) -> tokio::io::Result<()> {
        self.control_stream.write(b"220 KiraFTP v1.1.0\r\n").await?;
        Ok(())
    }
}
