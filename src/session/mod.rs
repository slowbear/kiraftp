// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

mod cwd;
mod features;
mod file_struct;
mod format;
mod info;
mod list;
mod login;
mod pwd;
mod quit;
mod receive;
mod send;
mod transfer_mode;
mod transfer_type;
mod unicode;
mod unknown_command;
mod wait;
mod welcome;

use crate::utils::config::Config;
use slog::{debug, warn, Logger};
use std::{cmp::min, collections::VecDeque, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

type IOResult = tokio::io::Result<()>;

enum TransferType {
    ASCII,
    Binary,
}

enum TransferMod {
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
    transfer_mode: TransferMod,
    transfer_type: TransferType,
    // 当前目录
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
            transfer_mode: TransferMod::Disable,
            transfer_type: TransferType::ASCII,
            current_path: config.path.clone(),
            logger,
            config,
        }
    }
    pub async fn run(&mut self) -> tokio::io::Result<()> {
        self.welcome().await?;
        let mut buffer = [0; 1024];
        let mut byte_stream = VecDeque::with_capacity(1024);
        let mut command = Vec::with_capacity(1024);
        loop {
            let mut pre = 0;
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
                if len == 0 {
                    return Ok(());
                }
                for &cur in buffer[0..len].iter() {
                    byte_stream.push_back(cur);
                }
            }
            if !command.ends_with(&[b'\r', b'\n']) {
                warn!(self.logger, "Unknown command recieved.");
                self.unknown_command().await?;
                continue;
            }
            // 移除CRLF
            command.pop();
            command.pop();
            // 统一大写处理
            let len = command.len();
            command[0..min(len, 4)].make_ascii_uppercase();
            let command = String::from_utf8_lossy(&command);
            debug!(self.logger, "Receive command: {}", command);
            match command.as_bytes() {
                b"PASV" => self.set_passive().await?,
                b"PWD" => self.print_working_directory().await?,
                b"FEAT" => self.list_features().await?,
                b"SYST" => self.print_info().await?,
                b"NOOP" => self.wait().await?,
                b"LIST" => self.list("").await?,
                b"OPTS UTF8 ON" => self.unicode().await?,
                b"QUIT" => return self.quit().await,
                _ => match command.split_once(' ') {
                    Some(("USER", para)) => self.pre_login(para).await?,
                    Some(("PASS", para)) => self.try_login(para).await?,
                    Some(("PORT", para)) => self.set_active(para).await?,
                    Some(("TYPE", para)) => self.set_transfer_type(para).await?,
                    Some(("MODE", para)) => self.set_transfer_mode(para).await?,
                    Some(("STRU", para)) => self.set_file_struct(para).await?,
                    Some(("CWD", para)) => self.change_working_directory(para).await?,
                    Some(("LIST", para)) => self.list(para).await?,
                    Some(("RETR", para)) => self.send(para).await?,
                    Some(("STOR", para)) => self.receive(para).await?,
                    _ => self.unknown_command().await?,
                },
            }
            // 执行单条指令后强制刷新缓冲区
            self.control_stream.flush().await?;
        }
    }
}
