// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub listen: IpAddr,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub path: PathBuf,
}

impl Config {
    pub fn address(&self) -> SocketAddr {
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
            path: PathBuf::from("/"),
        }
    }
}
