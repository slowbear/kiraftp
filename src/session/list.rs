use super::{FTPSession, IOResult, Transfer};
use path_absolutize::Absolutize;
use slog::{error, warn};
use std::{
    net::SocketAddr,
    os::unix::prelude::*,
    path::{Path, PathBuf},
};
use tokio::{
    fs::read_dir,
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
                let local = TcpSocket::new_v4()?;
                match local.bind(SocketAddr::new(ip.into(), port)) {
                    Ok(_) => match local.connect(*remote).await {
                        Ok(mut data_stream) => {
                            self.control_stream
                                .write(b"150 Here comes the directory listing.\r\n")
                                .await?;
                            if let Err(err) = self.list_inner(path, &mut data_stream).await {
                                warn!(self.logger, "Error during list: {}", err);
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
                            error!(self.logger, "Remote connection fail: {}", err);
                            return Err(err);
                        }
                    },
                    Err(err) => {
                        error!(self.logger, "Listen port {} failed: {}", port, err);
                        self.control_stream
                            .write(b"425 Server data connection close.\r\n")
                            .await?;
                    }
                }
            }
            Transfer::Passive(server) => match server.accept().await {
                Ok(_) => {}
                Err(_) => {}
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
    // list内部实现(未处理不存在路径)
    pub async fn list_inner(&mut self, path: &str, data_stream: &mut TcpStream) -> IOResult {
        let path = Path::new(path);
        let path = if path.is_absolute() {
            PathBuf::from(path)
        } else {
            Path::join(&self.current_path, path)
        };
        let mut dir = read_dir(path.absolutize_virtually(&self.virtual_root)?).await?;
        let mut text = Vec::<u8>::new();
        for item in dir.next_entry().await? {
            text.append(&mut item.file_name().as_bytes().to_vec());
            text.push(b' ');
        }
        text.push(b'\r');
        text.push(b'\n');
        data_stream.write(&text[..]).await?;
        Ok(())
    }
}
