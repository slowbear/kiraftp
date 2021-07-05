// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::SocketAddr;

pub fn parse_ipv4_addr(addr: &str) -> Option<SocketAddr> {
    let addr: Vec<&str> = addr.split(',').collect();
    if addr.len() != 6 || addr.iter().any(|x| x.parse::<u8>().is_err()) {
        return None;
    }
    let addr: Vec<u8> = addr.iter().map(|x| x.parse().unwrap()).collect();
    Some(SocketAddr::from((
        [addr[0], addr[1], addr[2], addr[3]],
        (addr[4] as u16) << 8 | (addr[5] as u16),
    )))
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
