use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub listen: IpAddr,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub path: String,
}

impl Config {
    pub fn get_addr(&self) -> SocketAddr {
        SocketAddr::new(self.listen, self.port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: IpAddr::from([0, 0, 0, 0]),
            port: 21,
            username: String::from("root"),
            password: String::from("password"),
            path: String::from("home"),
        }
    }
}
