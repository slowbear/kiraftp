use super::{FTPSession, IOResult, TransferMod};
use crate::utils::fs::display;
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
            TransferMod::Passive(server) => match server.accept().await {
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

    pub async fn list_inner(&mut self, path: &str, data_stream: &mut TcpStream) -> IOResult {
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
