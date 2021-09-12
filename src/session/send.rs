// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::{FTPSession, TransferMod, TransferType};
use slog::error;
use std::net::SocketAddr;
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpSocket, TcpStream},
};

impl FTPSession {
    pub async fn send(&mut self, path: &str) -> tokio::io::Result<()> {
        if !self.is_logged_in {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        match &self.transfer_mode {
            TransferMod::Active(remote) => {
                let (ip, port) = (self.config.listen, self.config.port - 1);
                let local = TcpSocket::new_v4()?;
                local.set_reuseaddr(true)?;
                match local.bind(SocketAddr::new(ip, port)) {
                    Ok(_) => match local.connect(*remote).await {
                        Ok(mut data_stream) => {
                            self.control_stream
                                .write(b"150 Opening BINARY mode data connection.\r\n")
                                .await?;
                            if let Err(err) = self.send_inner(path, &mut data_stream).await {
                                error!(self.logger, "Error during RETR: {}", err);
                                self.control_stream
                                    .write(b"426 Transfer aborted.\r\n")
                                    .await?;
                            } else {
                                self.control_stream
                                    .write(b"226 Transfer complete.\r\n")
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
                        .write(b"150 Opening BINARY mode data connection.\r\n")
                        .await?;
                    if let Err(err) = self.send_inner(path, &mut data_stream).await {
                        error!(self.logger, "Error during send: {}", err);
                        self.control_stream
                            .write(b"426 Transfer aborted.\r\n")
                            .await?;
                    } else {
                        self.control_stream
                            .write(b"226 Transfer complete.\r\n")
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

    pub async fn send_inner(
        &mut self,
        path: &str,
        data_stream: &mut TcpStream,
    ) -> tokio::io::Result<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(self.current_path.join(path))
            .await?;
        match self.transfer_type {
            TransferType::Ascii => {
                let mut buffer = [0; 32768];
                loop {
                    let len = file.read(&mut buffer).await?;
                    if len == 0 {
                        break;
                    }
                    for chunk in buffer[..len].split(|&x| x == b'\n') {
                        data_stream.write_all(chunk).await?;
                        data_stream.write_all(b"\r\n").await?;
                    }
                }
            }
            TransferType::Binary => {
                let mut buffer = [0; 32768];
                loop {
                    let len = file.read(&mut buffer).await?;
                    if len == 0 {
                        break;
                    }
                    data_stream.write(&buffer[..len]).await?;
                }
            }
        }
        Ok(())
    }
}
