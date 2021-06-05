use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub listen: Ipv4Addr,
    pub port: u16,
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn get_addr(&self) -> SocketAddr {
        SocketAddr::new(self.listen.into(), self.port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: Ipv4Addr::new(0, 0, 0, 0),
            port: 21,
            username: String::from("root"),
            password: String::from("password"),
        }
    }
}
