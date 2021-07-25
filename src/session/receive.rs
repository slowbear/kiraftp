// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use super::{FTPSession, IOResult, TransferMod, TransferType};
use slog::error;
use std::net::SocketAddr;
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpSocket, TcpStream},
};

impl FTPSession {
    pub async fn receive(&mut self, path: &str) -> IOResult {
        if !self.is_loggined {
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
                                .write(b"150 Ok to send data.\r\n")
                                .await?;
                            if let Err(err) = self.receive_inner(path, &mut data_stream).await {
                                error!(self.logger, "Error during receive: {}", err);
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
                        .write(b"150 Ok to send data.\r\n")
                        .await?;
                    if let Err(err) = self.receive_inner(path, &mut data_stream).await {
                        error!(self.logger, "Error during receive: {}", err);
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
        // 一次性链接
        self.transfer_mode = TransferMod::Disable;
        Ok(())
    }

    pub async fn receive_inner(&mut self, path: &str, data_stream: &mut TcpStream) -> IOResult {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.current_path.join(path))
            .await?;
        match self.transfer_type {
            TransferType::ASCII => {
                let mut buffer = [0u8; 32768];
                loop {
                    let len = data_stream.read(&mut buffer).await?;
                    if len == 0 {
                        break;
                    }
                    for &byte in buffer[..len].iter() {
                        if byte == b'\n' {
                            file.write_all(b"\r\n").await?;
                        } else {
                            file.write_all(&[byte]).await?;
                        }
                    }
                }
            }
            TransferType::Binary => {
                let mut buffer = [0u8; 32768];
                loop {
                    let len = data_stream.read(&mut buffer).await?;
                    if len == 0 {
                        break;
                    }
                    file.write_all(&buffer[..len]).await?;
                }
            }
        }
        file.flush().await?;
        Ok(())
    }
}
