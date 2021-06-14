use super::{FTPSession, IOResult, Transfer};
use crate::utils::fs::{combine, display, is_dir};
use slog::error;
use std::net::SocketAddr;
use tokio::{
    fs,
    io::AsyncWriteExt,
    net::{TcpSocket, TcpStream},
};

impl FTPSession {
    pub async fn list(&mut self, path: &str) -> IOResult {
        if !self.is_loggined {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        match &self.transfer {
            Transfer::Active(remote) => {
                let (ip, port) = (self.config.listen, self.config.port - 1);
                // 此处一般情况不可能Panic
                let local = TcpSocket::new_v4()?;
                local.set_reuseaddr(true)?;
                match local.bind(SocketAddr::new(ip, port)) {
                    Ok(_) => match local.connect(*remote).await {
                        Ok(mut data_stream) => {
                            self.control_stream
                                .write(b"150 Here comes the directory listing.\r\n")
                                .await?;
                            if let Err(err) = self.list_inner(path, &mut data_stream).await {
                                error!(self.logger, "Error during list: {}", err);
                                self.control_stream
                                    .write(b"426 Tansfer aborted.\r\n")
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
            Transfer::Passive(server) => match server.accept().await {
                Ok((mut data_stream, _)) => {
                    self.control_stream
                        .write(b"150 Here comes the directory listing.\r\n")
                        .await?;
                    if let Err(err) = self.list_inner(path, &mut data_stream).await {
                        error!(self.logger, "Error during list: {}", err);
                        self.control_stream
                            .write(b"426 Tansfer aborted.\r\n")
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
                        .write(b"426 Tansfer aborted.\r\n")
                        .await?;
                }
            },
            Transfer::Disable => {
                self.control_stream
                    .write(b"425 Use PORT or PASV first.\r\n")
                    .await?;
            }
        }
        // 一次性链接
        self.transfer = Transfer::Disable;
        Ok(())
    }

    // list内部实现
    pub async fn list_inner(&mut self, path: &str, data_stream: &mut TcpStream) -> IOResult {
        let path = match combine(&self.virtual_root, &self.current_path, path) {
            Some(path) => {
                if is_dir(&path).await {
                    path
                } else {
                    return Ok(());
                }
            }
            None => {
                return Ok(());
            }
        };
        let mut dir = fs::read_dir(path).await?;
        while let Some(item) = dir.next_entry().await? {
            if let Some(description) = display(&item).await {
                data_stream.write(description.as_bytes()).await?;
            }
        }
        Ok(())
    }
}
