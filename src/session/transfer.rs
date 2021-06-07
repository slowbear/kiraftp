use super::{FTPSession, IOResult, Transfer};
use crate::utils::helper::{parse_ipv4_addr, print_ipv4_addr};
use slog::debug;
use std::net::SocketAddr;
use tokio::{io::AsyncWriteExt, net::TcpListener};

impl FTPSession {
    // 主动模式
    pub async fn set_active(&mut self, remote: &str) -> IOResult {
        let remote = parse_ipv4_addr(remote);
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
        let listener = TcpListener::bind(SocketAddr::new(self.config.listen, 0)).await?;
        debug!(
            self.logger,
            "Try enter passive mode with {}",
            listener.local_addr()?
        );
        self.control_stream
            .write(
                format!(
                    "227 Entering Passive Mode {}.\r\n",
                    print_ipv4_addr(listener.local_addr()?)
                )
                .as_bytes(),
            )
            .await?;
        self.transfer = Transfer::Passive(listener);
        Ok(())
    }
}
