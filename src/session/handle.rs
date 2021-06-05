use std::net::SocketAddr;

use super::{feature, FTPSession, Transfer, TransferType};
use crate::utils::helper::{parse_addr, print_addr};
use slog::debug;
use tokio::{io::AsyncWriteExt, net::TcpListener};

impl FTPSession {
    pub async fn welcome(&mut self) -> tokio::io::Result<()> {
        self.control_stream.write(b"220 TryFTP v0.1.0\r\n").await?;
        Ok(())
    }
    pub async fn pre_login(&mut self, username: &str) -> tokio::io::Result<()> {
        self.control_stream
            .write(b"331 Please specify the password.\r\n")
            .await?;
        self.current_user = String::from(username);
        Ok(())
    }
    pub async fn try_login(&mut self, password: &str) -> tokio::io::Result<()> {
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
    pub async fn set_active(&mut self, addr: &str) -> tokio::io::Result<()> {
        let addr = parse_addr(addr);
        self.transfer = match addr {
            Some(addr) => {
                self.control_stream
                    .write(b"200 PORT command successful.\r\n")
                    .await?;
                Transfer::Active(addr)
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
    pub async fn set_passive(&mut self) -> tokio::io::Result<()> {
        // 由操作系统自动分配端口，一般不可能出错，故暂不处理Err
        let listener = TcpListener::bind(SocketAddr::new(self.config.listen.into(), 0)).await?;
        debug!(
            self.logger,
            "Try enter passive mode with {:?}",
            listener.local_addr().unwrap()
        );
        self.control_stream
            .write(
                format!(
                    "227 Entering Passive Mode {}.\r\n",
                    print_addr(listener.local_addr().unwrap())
                )
                .as_bytes(),
            )
            .await?;
        self.transfer = Transfer::Passive(listener);
        Ok(())
    }
    // 传输类型，支持ASCII码和二进制流
    pub async fn set_tranfer_type(&mut self, transfet_type: &str) -> tokio::io::Result<()> {
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
    pub async fn set_file_struct(&mut self, stru: &str) -> tokio::io::Result<()> {
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
    pub async fn set_tranfer_mode(&mut self, mode: &str) -> tokio::io::Result<()> {
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
    pub async fn list(&mut self, path: &str) -> tokio::io::Result<()> {
        // TODO: 列出当前目录文件
        Ok(())
    }
    pub async fn send(&mut self, path: &str) -> tokio::io::Result<()> {
        // TODO: 发送文件
        Ok(())
    }
    pub async fn recieve(&mut self, path: &str) -> tokio::io::Result<()> {
        // TODO: 接收文件
        Ok(())
    }
    pub async fn list_features(&mut self) -> tokio::io::Result<()> {
        self.control_stream.write(b"221-Features:\r\n").await?;
        for &item in feature::FEATURES {
            self.control_stream.write(item.as_bytes()).await?;
        }
        self.control_stream.write(b"221 End.\r\n").await?;
        Ok(())
    }
    pub async fn print_system_info(&mut self) -> tokio::io::Result<()> {
        self.control_stream.write(b"215 UNIX Type: L8\r\n").await?;
        Ok(())
    }
    pub async fn wait(&mut self) -> tokio::io::Result<()> {
        self.control_stream.write(b"200 NOOP ok.\r\n").await?;
        Ok(())
    }
    pub async fn disconnect(&mut self) -> tokio::io::Result<()> {
        self.control_stream.write(b"221 Goodbye.\r\n").await?;
        Ok(())
    }
    pub async fn unknow_command(&mut self) -> tokio::io::Result<()> {
        self.control_stream
            .write(b"500 Unknow command.\r\n")
            .await?;
        Ok(())
    }
}
