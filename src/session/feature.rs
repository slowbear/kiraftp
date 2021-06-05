use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

pub const FEATURES: &[&'static str] = &[" PASV\r\n", " UTF8\r\n"];

impl FTPSession {
    pub async fn list_features(&mut self) -> IOResult {
        self.control_stream.write(b"221-Features:\r\n").await?;
        for &item in FEATURES {
            self.control_stream.write(item.as_bytes()).await?;
        }
        self.control_stream.write(b"221 End.\r\n").await?;
        Ok(())
    }
}
