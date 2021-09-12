// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::{FTPSession, TransferType};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn set_transfer_type(&mut self, transfer_type: &str) -> tokio::io::Result<()> {
        let transfer_type = transfer_type.to_uppercase();
        if transfer_type == "A" {
            self.transfer_type = TransferType::Ascii;
            self.control_stream
                .write(b"200 Switching to ASCII mode.\r\n")
                .await?;
        } else if transfer_type == "I" {
            self.transfer_type = TransferType::Binary;
            self.control_stream
                .write(b"200 Switching to Binary mode.\r\n")
                .await?;
        } else {
            self.control_stream
                .write(b"504 Unsupported type.\r\n")
                .await?;
        }
        Ok(())
    }
}
