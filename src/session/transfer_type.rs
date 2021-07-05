// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::{FTPSession, IOResult, TransferType};
use tokio::io::AsyncWriteExt;

impl FTPSession {
    // 传输类型，支持ASCII码和二进制流
    pub async fn set_tranfer_type(&mut self, transfet_type: &str) -> IOResult {
        let transfet_type = transfet_type.to_uppercase();
        if transfet_type == "A" {
            self.transfer_type = TransferType::ASCII;
            self.control_stream
                .write(b"200 Switching to ASCII mode.\r\n")
                .await?;
        } else if transfet_type == "I" {
            self.transfer_type = TransferType::Binary;
            self.control_stream
                .write(b"200 Switching to Binary mode.\r\n")
                .await?;
        } else {
            self.control_stream
                .write(b"504 Unsupport type.\r\n")
                .await?;
        }
        Ok(())
    }
}
