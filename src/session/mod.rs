mod feature;
mod file_mode;
mod info;
mod list;
mod login;
mod pwd;
mod quit;
mod recieve;
mod send;
mod transfer;
mod transfer_mode;
mod transfer_type;
mod unknow_command;
mod wait;
mod welcome;

use crate::utils::Config;
use slog::{warn, Logger};
use std::{
    env::current_dir,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

type IOResult = tokio::io::Result<()>;

enum TransferType {
    ASCII,
    Binary,
}

enum Transfer {
    Active(SocketAddr),
    Passive(TcpListener),
    Disable,
}

pub struct FTPSession {
    // 控制连接参数
    control_stream: TcpStream,
    current_user: String,
    is_loggined: bool,
    is_anonymous: bool,
    // 数据连接参数
    transfer: Transfer,
    transfer_type: TransferType,
    // 目录树
    virtual_root: PathBuf,
    current_path: PathBuf,
    // 会话其他参数
    pub logger: Arc<Logger>,
    pub config: Arc<Config>,
}

impl FTPSession {
    pub fn new(control_stream: TcpStream, logger: Arc<Logger>, config: Arc<Config>) -> Self {
        Self {
            control_stream,
            current_user: String::new(),
            is_loggined: false,
            is_anonymous: false,
            transfer: Transfer::Disable,
            transfer_type: TransferType::Binary,
            // 仅在执行文件所在文件夹被删除或无权限访问时Panic
            virtual_root: Path::join(current_dir().unwrap().as_path(), config.path.clone()),
            current_path: PathBuf::new(),
            logger,
            config,
        }
    }
    pub async fn run(&mut self) -> tokio::io::Result<()> {
        self.welcome().await?;
        let mut command_buffer = [0u8; 256];
        loop {
            let len = self.control_stream.read(&mut command_buffer).await?;
            let command_buffer = &mut command_buffer[0..len];
            if !command_buffer.ends_with(&[b'\r', b'\n']) {
                warn!(self.logger, "Unknown data recieve.");
                self.unknow_command().await?;
                continue;
            }
            if len >= 6 {
                command_buffer[0..4].make_ascii_uppercase();
            }
            let command = String::from_utf8_lossy(&command_buffer);
            match command.split_once(|sep: char| sep.is_whitespace()) {
                Some(("USER", para)) => self.pre_login(para.trim_end()).await?,
                Some(("PASS", para)) => self.try_login(para.trim_end()).await?,
                Some(("PORT", para)) => self.set_active(para.trim_end()).await?,
                Some(("PASV", _)) => self.set_passive().await?,
                Some(("TYPE", para)) => self.set_tranfer_type(para.trim_end()).await?,
                Some(("MODE", para)) => self.set_tranfer_mode(para.trim_end()).await?,
                Some(("STRU", para)) => self.set_file_struct(para.trim_end()).await?,
                // 以下命令需要登录
                Some(("PWD", _)) => self.print_working_directory().await?,
                // TODO: 更改当前目录
                Some(("CWD", para)) => {}
                // TODO: 列出当前目录文件
                Some(("LIST", para)) => self.list(para.trim_end()).await?,
                // TODO: 下载文件
                Some(("RETR", para)) => self.send(para.trim_end()).await?,
                // TODO: 上传文件
                Some(("STOR", para)) => self.recieve(para.trim_end()).await?,
                // 以上命令需要登录
                Some(("FEAT", _)) => self.list_features().await?,
                Some(("SYST", _)) => self.print_info().await?,
                Some(("NOOP", _)) => self.wait().await?,
                Some(("QUIT", _)) => {
                    self.quit().await?;
                    break;
                }
                _ => self.unknow_command().await?,
            }
            // 执行单条指令后强制刷新缓冲区
            self.control_stream.flush().await?;
        }
        Ok(())
    }
}
