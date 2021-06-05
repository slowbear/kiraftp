use super::{feature, FTPSession, Transfer, TransferType};
use crate::utils::helper::{parse_addr, print_addr};
use path_absolutize::Absolutize;
use slog::{debug, error, warn};
use std::{
    net::SocketAddr,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};
use tokio::{
    fs::read_dir,
    io::AsyncWriteExt,
    net::{TcpListener, TcpSocket, TcpStream},
};

type IOResult = tokio::io::Result<()>;

impl FTPSession {
    pub async fn welcome(&mut self) -> IOResult {
        self.control_stream.write(b"220 TryFTP v0.1.0\r\n").await?;
        Ok(())
    }
    pub async fn pre_login(&mut self, username: &str) -> IOResult {
        self.control_stream
            .write(b"331 Please specify the password.\r\n")
            .await?;
        self.current_user = String::from(username);
        Ok(())
    }
    pub async fn try_login(&mut self, password: &str) -> IOResult {
        if self.current_user == "anonymous" {
            self.control_stream
                .write(b"230 Login successful.\r\n")
                .await?;
            self.is_anonymous = true;
            self.is_loggined = true;
        } else if self.current_user == self.config.username && password == self.config.password {
            self.control_stream
                .write(b"230 Login successful.\r\n")
                .await?;
            self.is_loggined = true;
        } else {
            self.control_stream
                .write(b"530 Login incorrect.\r\n")
                .await?;
        }
        Ok(())
    }
    // 主动模式
    pub async fn set_active(&mut self, remote: &str) -> IOResult {
        let remote = parse_addr(remote);
        self.transfer = match remote {
            Some(remote) => {
                self.control_stream
                    .write(b"200 PORT command successful.\r\n")
                    .await?;
                Transfer::Active(remote)
            }
            None => {
                self.control_stream
                    .write(b"501 Illegal address.\r\n")
                    .await?;
                Transfer::Disable
            }
        };
        Ok(())
    }
    // 被动模式
    pub async fn set_passive(&mut self) -> IOResult {
        let listener = TcpListener::bind(SocketAddr::new(self.config.listen.into(), 0)).await?;
        debug!(
            self.logger,
            "Try enter passive mode with {}",
            listener.local_addr()?
        );
        self.control_stream
            .write(
                format!(
                    "227 Entering Passive Mode {}.\r\n",
                    print_addr(listener.local_addr()?)
                )
                .as_bytes(),
            )
            .await?;
        self.transfer = Transfer::Passive(listener);
        Ok(())
    }
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
    // 只支持F(File)结构，与主流FTP服务器一至
    pub async fn set_file_struct(&mut self, stru: &str) -> IOResult {
        self.control_stream
            .write(if stru == "F" {
                b"200 Structure set to F.\r\n"
            } else {
                b"504 Bad STRU command.\r\n"
            })
            .await?;
        Ok(())
    }
    // 只支持S(Stream)模式，简化实现
    pub async fn set_tranfer_mode(&mut self, mode: &str) -> IOResult {
        let mode = mode.to_ascii_uppercase();
        self.control_stream
            .write(if mode == "S" {
                b"200 Mode set to S.\r\n"
            } else {
                b"504 Bad MODE command.\r\n"
            })
            .await?;
        Ok(())
    }
    // list内部实现，不存在路径当作空文件夹处理
    pub async fn list_inner(&mut self, path: &str, data_stream: &mut TcpStream) -> IOResult {
        let path = Path::new(path);
        let real_path = if path.is_absolute() {
            Path::join(&self.current_path, path)
        } else {
            PathBuf::from(path)
        };
        let mut dir = read_dir(real_path.absolutize_virtually(&self.virtual_root)?).await?;
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
    pub async fn list(&mut self, path: &str) -> IOResult {
        if !self.is_loggined {
            self.control_stream
                .write(b"530 Please login with USER and PASS.\r\n")
                .await?;
            return Ok(());
        }
        match self.transfer {
            Transfer::Active(remote) => {
                let (ip, port) = (self.config.listen, self.config.port + 1);
                let local = TcpSocket::new_v4()?;
                match local.bind(SocketAddr::new(ip.into(), port)) {
                    Ok(_) => match local.connect(remote).await {
                        Ok(mut data_stream) => {
                            self.control_stream
                                .write(b"150 Here comes the directory listing.\r\n")
                                .await?;
                            if let Err(_) = self.list_inner(path, &mut data_stream).await {
                                warn!(self.logger, "Invalid path {} accessd.", path);
                            }
                            self.control_stream
                                .write(b"226 Directory send OK.\r\n")
                                .await?;
                        }
                        Err(err) => {
                            error!(self.logger, "Remote connection fail: {}", err);
                            self.control_stream.write(b"\r\n").await?;
                            return Ok(());
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
            Transfer::Passive(_) => {}
            Transfer::Disable => {
                self.control_stream
                    .write(b"425 Use PORT or PASV first.\r\n")
                    .await?;
            }
        }
        Ok(())
    }
    pub async fn send(&mut self, path: &str) -> IOResult {
        // TODO: 发送文件
        Ok(())
    }
    pub async fn recieve(&mut self, path: &str) -> IOResult {
        // TODO: 接收文件
        Ok(())
    }
    pub async fn list_features(&mut self) -> IOResult {
        self.control_stream.write(b"221-Features:\r\n").await?;
        for &item in feature::FEATURES {
            self.control_stream.write(item.as_bytes()).await?;
        }
        self.control_stream.write(b"221 End.\r\n").await?;
        Ok(())
    }
    pub async fn print_system_info(&mut self) -> IOResult {
        self.control_stream.write(b"215 UNIX Type: L8\r\n").await?;
        Ok(())
    }
    pub async fn wait(&mut self) -> IOResult {
        self.control_stream.write(b"200 NOOP ok.\r\n").await?;
        Ok(())
    }
    pub async fn disconnect(&mut self) -> IOResult {
        self.control_stream.write(b"221 Goodbye.\r\n").await?;
        Ok(())
    }
    pub async fn unknow_command(&mut self) -> IOResult {
        self.control_stream
            .write(b"500 Unknow command.\r\n")
            .await?;
        Ok(())
    }
}
