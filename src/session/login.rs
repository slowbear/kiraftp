use super::{FTPSession, IOResult};
use tokio::io::AsyncWriteExt;

impl FTPSession {
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
}
