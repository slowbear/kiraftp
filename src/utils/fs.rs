// Copyright 2021 Slowy <slowyfine@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

use chrono::{DateTime, Local};
use libc::*;
use std::{os::unix::prelude::*, path::Path};
use tokio::fs::{self as tokiofs, DirEntry};

#[inline(always)]
pub async fn is_dir(path: impl AsRef<Path>) -> bool {
    tokiofs::metadata(path)
        .await
        .map(|meta| meta.is_dir())
        .unwrap_or(false)
}

pub async fn display(item: &DirEntry) -> Option<String> {
    match item.metadata().await {
        Ok(metadata) => {
            let mode = parse_permissions(metadata.permissions().mode());
            let nlink = metadata.nlink();
            let user = users::get_user_by_uid(metadata.uid())?;
            let user = user.name().to_string_lossy();
            let group = users::get_group_by_gid(metadata.gid())?;
            let group = group.name().to_string_lossy();
            let size = metadata.size();
            let modified = DateTime::<Local>::from(metadata.modified().unwrap())
                .format("%b %d %H:%M")
                .to_string();
            let filename = item.file_name().into_string().unwrap();
            Some(if filename.contains(' ') {
                format!(
                    "{} {} {} {} {} {} '{}'\r\n",
                    mode, nlink, user, group, size, modified, filename
                )
            } else {
                format!(
                    "{} {} {} {} {} {} {}\r\n",
                    mode, nlink, user, group, size, modified, filename
                )
            })
        }
        Err(_) => None,
    }
}

fn parse_permissions(mode: u32) -> String {
    let prop = file_type(mode);
    let user = triplet(mode, S_IRUSR, S_IWUSR, S_IXUSR);
    let group = triplet(mode, S_IRGRP, S_IWGRP, S_IXGRP);
    let other = triplet(mode, S_IROTH, S_IWOTH, S_IXOTH);
    [prop, user, group, other].join("")
}

fn triplet(mode: u32, read: u32, write: u32, execute: u32) -> &'static str {
    match (mode & read, mode & write, mode & execute) {
        (0, 0, 0) => "---",
        (_, 0, 0) => "r--",
        (0, _, 0) => "-w-",
        (0, 0, _) => "--x",
        (_, 0, _) => "r-x",
        (_, _, 0) => "rw-",
        (0, _, _) => "-wx",
        (_, _, _) => "rwx",
    }
}

fn file_type(mode: u32) -> &'static str {
    match mode & S_IFMT {
        S_IFDIR => "d",
        S_IFLNK => "l",
        S_IFSOCK => "s",
        S_IFBLK => "b",
        S_IFCHR => "c",
        S_IFIFO => "p",
        _ => "-",
    }
}
