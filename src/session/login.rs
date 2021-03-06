// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::FTPSession;
use tokio::io::AsyncWriteExt;

impl FTPSession {
    pub async fn pre_login(&mut self, username: &str) -> tokio::io::Result<()> {
        if self.is_logged_in {
            self.control_stream
                .write(b"530 Can't change to another user.\r\n")
                .await?;
        } else {
            self.control_stream
                .write(b"331 Please specify the password.\r\n")
                .await?;
            self.current_user = String::from(username);
        }
        Ok(())
    }

    pub async fn try_login(&mut self, password: &str) -> tokio::io::Result<()> {
        if self.is_logged_in {
            self.control_stream
                .write(b"230 Already logged in.\r\n")
                .await?;
        } else if self.current_user == "anonymous" {
            self.control_stream
                .write(b"230 Login successfully.\r\n")
                .await?;
            self.is_anonymous = true;
            self.is_logged_in = true;
        } else if self.current_user == self.config.username && password == self.config.password {
            self.control_stream
                .write(b"230 Login successful.\r\n")
                .await?;
            self.is_logged_in = true;
        } else {
            self.control_stream
                .write(b"530 Login incorrect.\r\n")
                .await?;
        }
        Ok(())
    }
}
