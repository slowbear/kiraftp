use chrono::{DateTime, Local};
use libc::*;
use std::{
    os::unix::prelude::*,
    path::{Path, PathBuf},
};
use tokio::fs::{self, DirEntry};

#[inline(always)]
pub async fn is_dir(path: impl AsRef<Path>) -> bool {
    fs::metadata(&path)
        .await
        .map(|meta| meta.is_dir())
        .unwrap_or(false)
}

pub fn combine(root: &PathBuf, current: &PathBuf, extra: &str) -> Option<PathBuf> {
    let extra = Path::new(extra);
    let path = {
        let mut path = PathBuf::from(root);
        if extra.is_absolute() {
            path.push(extra);
        } else {
            path.push(Path::join(current, extra));
        }
        path
    };
    match path.canonicalize() {
        Ok(path) => Some(path),
        Err(_) => None,
    }
}

pub async fn display(item: &DirEntry) -> Option<String> {
    match item.metadata().await {
        Ok(metadata) => {
            let size = metadata.size();
            // 仅Unix支持获取修改时间
            let modified: DateTime<Local> = DateTime::from(metadata.modified().unwrap());
            let mode = parse_permissions(metadata.permissions().mode());
            let name = item.file_name().into_string().unwrap();
            Some(format!(
                "{} {:>10} {} {}\r\n",
                mode,
                size,
                modified.format("%b %d %H:%M").to_string(),
                name
            ))
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

#[inline(always)]
fn triplet(mode: u32, read: u32, write: u32, execute: u32) -> String {
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
    .to_string()
}

#[inline(always)]
fn file_type(mode: u32) -> String {
    if mode & S_IFMT == S_IFDIR {
        "d"
    } else if mode & S_IFMT == S_IFLNK {
        "l"
    } else if mode & S_IFMT == S_IFSOCK {
        "s"
    } else if mode & S_IFMT == S_IFBLK {
        "b"
    } else if mode & S_IFMT == S_IFCHR {
        "c"
    } else if mode & S_IFMT == S_IFIFO {
        "p"
    } else {
        "-"
    }
    .to_string()
}
