// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn print_info(&mut self) -> IOResult {
        self.control_stream.write(b"215 UNIX Type: L8\r\n").await?;
        Ok(())
    }
}
