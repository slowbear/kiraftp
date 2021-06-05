mod session;
mod utils;

use session::FTPSession;
use slog::{debug, error, info, o, warn, Drain, Logger};
use slog_async::Async;
use slog_term::{FullFormat, TermDecorator};
use std::sync::Arc;
use tokio::net::TcpListener;
use utils::Config;

#[tokio::main]
async fn main() {
    // 初始化日志
    let decorator = TermDecorator::new().build();
    let drain = FullFormat::new(decorator).build().fuse();
    let drain = Async::new(drain).build().fuse();
    // 传递引用进子进程需要'static生命周期，暂用Arc代替
    let logger = Arc::new(Logger::root(drain, o!()));
    info!(logger, "Log started!");
    // 初始化配置
    let config = match tokio::fs::read_to_string("config.yaml").await {
        Ok(config) => match serde_yaml::from_str(config.as_str()) {
            Ok(config) => config,
            Err(_) => {
                warn!(
                    logger,
                    "Parse config.yaml failed, fallback to default config."
                );
                Config::default()
            }
        },
        Err(_) => Config::default(),
    };
    debug!(logger, "Listening {}:{}", config.listen, config.port);
    let config = Arc::new(config);
    // 启动服务器
    match TcpListener::bind(config.get_addr()).await {
        Ok(server) => loop {
            match server.accept().await {
                Ok((stream, remote)) => {
                    info!(logger, "Connection from {} established.", remote.ip());
                    let mut session = FTPSession::new(stream, logger.clone(), config.clone());
                    tokio::spawn(async move {
                        match session.run().await {
                            Ok(_) => {
                                info!(session.logger, "Connection from {} closed.", remote.ip())
                            }
                            Err(err) => {
                                error!(session.logger, "Error during writing socket: {}", err)
                            }
                        }
                    });
                }
                Err(err) => {
                    error!(logger, "Unexpected connection: {}", err);
                }
            }
        },
        Err(err) => {
            error!(logger, "Listen fail: {}", err);
        }
    }
}
