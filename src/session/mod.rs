mod cwd;
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

use crate::utils::config::Config;
use slog::{debug, warn, Logger};
use std::{
    collections::VecDeque,
    env,
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
            virtual_root: Path::join(env::current_dir().unwrap().as_path(), config.path.clone()),
            current_path: PathBuf::new(),
            logger,
            config,
        }
    }
    pub async fn run(&mut self) -> tokio::io::Result<()> {
        self.welcome().await?;
        let mut buffer = [0u8; 1024];
        let mut byte_stream = VecDeque::<u8>::new();
        let mut command = Vec::<u8>::with_capacity(1024);
        loop {
            let mut pre = b' ';
            command.clear();
            loop {
                while !byte_stream.is_empty() {
                    let &cur = byte_stream.front().unwrap();
                    byte_stream.pop_front();
                    command.push(cur);
                    if (pre, cur) == (b'\r', b'\n') {
                        break;
                    }
                    pre = cur;
                }
                if command.ends_with(&[b'\r', b'\n']) || command.len() > 1024 {
                    break;
                }
                let len = self.control_stream.read(&mut buffer).await?;
                // 控制连接已关闭
                if len == 0 {
                    return Ok(());
                }
                for &cur in buffer[0..len].iter() {
                    byte_stream.push_back(cur);
                }
            }
            if !command.ends_with(&[b'\r', b'\n']) {
                warn!(self.logger, "Unknown command recieved.");
                self.unknow_command().await?;
                continue;
            }
            // 移除CRLF
            command.pop();
            command.pop();
            // 统一大写处理
            if command.len() > 4 {
                command[0..4].make_ascii_uppercase();
            } else {
                command.make_ascii_uppercase();
            }
            let command = String::from_utf8_lossy(&command);
            debug!(self.logger, "Recieve command: {}", command);
            match command.as_bytes() {
                // 无参数指令
                b"PASV" => self.set_passive().await?,
                b"PWD" => self.print_working_directory().await?,
                b"FEAT" => self.list_features().await?,
                b"SYST" => self.print_info().await?,
                b"NOOP" => self.wait().await?,
                b"LIST" => self.list("").await?,
                b"QUIT" => {
                    self.quit().await?;
                    break;
                }
                // 带参数指令
                _ => {
                    match command.split_once(' ') {
                        Some(("USER", para)) => self.pre_login(para).await?,
                        Some(("PASS", para)) => self.try_login(para).await?,
                        Some(("PORT", para)) => self.set_active(para).await?,
                        Some(("TYPE", para)) => self.set_tranfer_type(para).await?,
                        Some(("MODE", para)) => self.set_tranfer_mode(para).await?,
                        Some(("STRU", para)) => self.set_file_struct(para).await?,
                        // 以下命令需要登录
                        Some(("CWD", para)) => self.change_working_directory(para).await?,
                        Some(("LIST", para)) => self.list(para).await?,
                        // TODO: 下载文件
                        Some(("RETR", para)) => self.send(para).await?,
                        // TODO: 上传文件
                        Some(("STOR", para)) => self.recieve(para).await?,
                        // 以上命令需要登录
                        _ => self.unknow_command().await?,
                    }
                }
            }
            // 执行单条指令后强制刷新缓冲区
            self.control_stream.flush().await?;
        }
        Ok(())
    }
}
