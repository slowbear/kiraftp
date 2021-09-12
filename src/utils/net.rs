// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::SocketAddr;

pub fn parse_ipv4_addr(addr: impl AsRef<str>) -> Option<SocketAddr> {
    let addr: Vec<&str> = addr.as_ref().split(',').collect();
    let addr: Result<Vec<u8>, _> = addr.iter().map(|x| x.parse()).collect();
    match addr {
        Ok(addr) => Some(SocketAddr::from((
            [addr[0], addr[1], addr[2], addr[3]],
            (addr[4] as u16) << 8 | (addr[5] as u16),
        ))),
        Err(_) => None,
    }
}

pub fn print_ipv4_addr(addr: SocketAddr) -> String {
    let (ip, port) = (addr.ip(), addr.port());
    format!(
        "({},{},{})",
        ip.to_string().replace('.', ","),
        port >> 8,
        port & 0xff
    )
}
