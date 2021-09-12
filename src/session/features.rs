// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::FTPSession;
use tokio::io::AsyncWriteExt;

pub const FEATURES: &[&str] = &["PASV\r\n", "UTF8\r\n"];

impl FTPSession {
    pub async fn list_features(&mut self) -> tokio::io::Result<()> {
        self.control_stream.write(b"221 Features:\r\n").await?;
        for &item in FEATURES {
            self.control_stream.write(item.as_bytes()).await?;
        }
        self.control_stream.write(b"221 End\r\n").await?;
        Ok(())
    }
}
