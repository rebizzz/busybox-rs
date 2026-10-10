use super::common::*;
use crate::core::{Applet, Result};
use std::collections::HashMap;
use std::ffi::{CStr, CString, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};

pub struct TreeApplet;

impl Applet for TreeApplet {
    fn name(&self) -> &'static str {
        "tree"
    }
    fn description(&self) -> &'static str {
        "List contents of directories in a tree-like format"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut all_files = false;
        let mut dirs_only = false;
        let mut root_dir: Option<&Path> = None;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-a" {
                all_files = true;
            } else if b == b"-d" {
                dirs_only = true;
            } else if !b.is_empty() && b[0] == b'-' {
            } else if root_dir.is_none() {
                root_dir = Some(Path::new(arg));
            }
        }

        let root = root_dir.unwrap_or_else(|| Path::new("."));
        let mut out = Vec::new();
        out.extend_from_slice(root.as_os_str().as_bytes());
        out.push(b'\n');

        let mut dir_count = 0u64;
        let mut file_count = 0u64;
        let mut prefix = Vec::new();

        traverse_tree(
            root,
            all_files,
            dirs_only,
            &mut prefix,
            &mut dir_count,
            &mut file_count,
            &mut out,
        );

        out.push(b'\n');
        put_num(&mut out, dir_count);
        out.extend_from_slice(if dir_count == 1 {
            b" directory"
        } else {
            b" directories"
        });
        if !dirs_only {
            out.extend_from_slice(b", ");
            put_num(&mut out, file_count);
            out.extend_from_slice(if file_count == 1 {
                b" file\n"
            } else {
                b" files\n"
            });
        } else {
            out.push(b'\n');
        }

        print_bytes(&out);
        Ok(0)
    }
}

fn traverse_tree(
    dir: &Path,
    all_files: bool,
    dirs_only: bool,
    prefix: &mut Vec<u8>,
    dir_count: &mut u64,
    file_count: &mut u64,
    out: &mut Vec<u8>,
) {
    let mut entries = Vec::new();
    if let Ok(rd) = fs::read_dir(dir) {
        for entry in rd.flatten() {
            let file_name = entry.file_name();
            let b = file_name.as_bytes();
            if !all_files && b.starts_with(b".") {
                continue;
            }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if dirs_only && !is_dir {
                continue;
            }
            entries.push((entry.path(), file_name, is_dir));
        }
    }

    entries.sort_by(|a, b| a.1.cmp(&b.1));

    let len = entries.len();
    for (i, (path, name, is_dir)) in entries.into_iter().enumerate() {
        let is_last = i + 1 == len;
        out.extend_from_slice(prefix);
        if is_last {
            out.extend_from_slice("└── ".as_bytes());
        } else {
            out.extend_from_slice("├── ".as_bytes());
        }
        out.extend_from_slice(name.as_bytes());
        out.push(b'\n');

        if is_dir {
            *dir_count += 1;
            let prev_len = prefix.len();
            if is_last {
                prefix.extend_from_slice(b"    ");
            } else {
                prefix.extend_from_slice("│   ".as_bytes());
            }
            traverse_tree(
                &path, all_files, dirs_only, prefix, dir_count, file_count, out,
            );
            prefix.truncate(prev_len);
        } else {
            *file_count += 1;
        }
    }
}
