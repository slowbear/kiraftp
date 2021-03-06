// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::{FTPSession, TransferMod};
use crate::utils::net::{parse_ipv4_addr, print_ipv4_addr};
use slog::debug;
use std::net::SocketAddr;
use tokio::{io::AsyncWriteExt, net::TcpListener};

impl FTPSession {
    pub async fn set_active(&mut self, remote: &str) -> tokio::io::Result<()> {
        if !self.is_logged_in {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        let remote = parse_ipv4_addr(remote);
        self.transfer_mode = match remote {
            Some(remote) => {
                debug!(self.logger, "Try entering active mode with {}", remote);
                self.control_stream
                    .write(b"200 PORT command successful.\r\n")
                    .await?;
                TransferMod::Active(remote)
            }
            None => {
                self.control_stream
                    .write(b"501 Illegal address.\r\n")
                    .await?;
                TransferMod::Disable
            }
        };
        Ok(())
    }

    pub async fn set_passive(&mut self) -> tokio::io::Result<()> {
        if !self.is_logged_in {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        self.transfer_mode = match TcpListener::bind(SocketAddr::new(self.config.listen, 0)).await {
            Ok(listener) => {
                debug!(
                    self.logger,
                    "Try entering passive mode with {}",
                    listener.local_addr()?
                );
                self.control_stream
                    .write(
                        format!(
                            "227 Entering Passive Mode {}.\r\n",
                            print_ipv4_addr(listener.local_addr()?)
                        )
                        .as_bytes(),
                    )
                    .await?;
                TransferMod::Passive(listener)
            }
            Err(err) => {
                debug!(self.logger, "Create socket unsuccessfully: {}", err);
                self.control_stream
                    .write(b"421 Could not create socket.\r\n")
                    .await?;
                TransferMod::Disable
            }
        };
        Ok(())
    }
}
