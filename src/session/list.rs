// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::{FTPSession, TransferMod};
use crate::utils::fs::display;
use slog::error;
use std::net::SocketAddr;
use tokio::{
    fs,
    io::AsyncWriteExt,
    net::{TcpSocket, TcpStream},
};

impl FTPSession {
    pub async fn list(&mut self, opts: &str) -> tokio::io::Result<()> {
        if !self.is_logged_in {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        // Hack for nautils, ignore all options.
        let path = opts.split(' ').find(|x| !x.starts_with('-')).unwrap_or("");
        match &self.transfer_mode {
            TransferMod::Active(remote) => {
                let (ip, port) = (self.config.listen, self.config.port - 1);
                let local = TcpSocket::new_v4()?;
                local.set_reuseaddr(true)?;
                match local.bind(SocketAddr::new(ip, port)) {
                    Ok(_) => match local.connect(*remote).await {
                        Ok(mut data_stream) => {
                            self.control_stream
                                .write(b"150 Here comes the directory listing.\r\n")
                                .await?;
                            if let Err(err) = self.list_inner(path, &mut data_stream).await {
                                error!(self.logger, "Error during LIST: {}", err);
                                self.control_stream
                                    .write(b"426 Transfer aborted.\r\n")
                                    .await?;
                            } else {
                                self.control_stream
                                    .write(b"226 Directory send OK.\r\n")
                                    .await?;
                            }
                        }
                        Err(err) => {
                            error!(self.logger, "Failed to connect to remote: {}", err);
                            self.control_stream
                                .write(b"425 Can't open data connection.\r\n")
                                .await?;
                        }
                    },
                    Err(err) => {
                        error!(self.logger, "Failed to listening port {}: {}", port, err);
                        self.control_stream
                            .write(b"425 Server data connection close.\r\n")
                            .await?;
                    }
                }
            }
            TransferMod::Passive(server) => match server.accept().await {
                Ok((mut data_stream, _)) => {
                    self.control_stream
                        .write(b"150 Here comes the directory listing.\r\n")
                        .await?;
                    if let Err(err) = self.list_inner(path, &mut data_stream).await {
                        error!(self.logger, "Error during list: {}", err);
                        self.control_stream
                            .write(b"426 Transfer aborted.\r\n")
                            .await?;
                    } else {
                        self.control_stream
                            .write(b"226 Directory send OK.\r\n")
                            .await?;
                    }
                }
                Err(err) => {
                    error!(self.logger, "Unexpected data connection: {}", err);
                    self.control_stream
                        .write(b"426 Transfer aborted.\r\n")
                        .await?;
                }
            },
            TransferMod::Disable => {
                self.control_stream
                    .write(b"425 Use PORT or PASV first.\r\n")
                    .await?;
            }
        }
        self.transfer_mode = TransferMod::Disable;
        Ok(())
    }

    pub async fn list_inner(
        &mut self,
        path: &str,
        data_stream: &mut TcpStream,
    ) -> tokio::io::Result<()> {
        let path = self.current_path.join(path);
        let mut dir = fs::read_dir(path).await?;
        while let Some(item) = dir.next_entry().await? {
            if let Some(description) = display(&item).await {
                data_stream.write(description.as_bytes()).await?;
            }
        }
        Ok(())
    }
}
