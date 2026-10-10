use crate::core::{Applet, Result};
use crate::core::digest::{BsdSum, Digest, Md5, Sha1, Sha256, Sha512, SysVSum};
use crate::core::fs::{open_or_stdin, read_bytes_or_stdin};
use super::common::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::{CString, OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime};
use std::env;

pub struct StringsApplet;
impl Applet for StringsApplet {
    fn name(&self) -> &'static str {
        "strings"
    }
    fn description(&self) -> &'static str {
        "Find and print letter sequences in binary files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut min_len = 4;
        let mut print_filename = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-n" {
                if i + 1 < args.len() {
                    min_len = args[i + 1].to_string_lossy().parse().unwrap_or(4);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-n") {
                min_len = arg.to_string_lossy()[2..].parse().unwrap_or(4);
            } else if bytes.starts_with(b"-")
                && bytes.len() > 1
                && bytes[1..].iter().all(|b| b.is_ascii_digit())
            {
                min_len = arg.to_string_lossy()[1..].parse().unwrap_or(4);
            } else if bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                for &b in &bytes[1..] {
                    match b {
                        b'a' => {}
                        b'f' => print_filename = true,
                        _ => {}
                    }
                }
            } else {
                files.push(Path::new(arg));
            }
            i += 1;
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for file in files {
            let file_prefix = if print_filename {
                format!("{}: ", file.display())
            } else {
                String::new()
            };
            let content = read_bytes_or_stdin(file)?;
            let mut cur = Vec::new();
            for &b in &content {
                if (32..127).contains(&b) || b == b'\t' {
                    cur.push(b);
                } else {
                    if cur.len() >= min_len {
                        if print_filename {
                            handle.write_all(file_prefix.as_bytes())?;
                        }
                        handle.write_all(&cur)?;
                        handle.write_all(b"\n")?;
                    }
                    cur.clear();
                }
            }
            if cur.len() >= min_len {
                if print_filename {
                    if let Err(e) = handle.write_all(file_prefix.as_bytes()) {
                        if e.kind() == io::ErrorKind::BrokenPipe {
                            return Ok(0);
                        }
                        return Err(e.into());
                    }
                }
                if let Err(e) = handle.write_all(&cur) {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        return Ok(0);
                    }
                    return Err(e.into());
                }
                if let Err(e) = handle.write_all(b"\n") {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        return Ok(0);
                    }
                    return Err(e.into());
                }
            }
        }
        Ok(0)
    }
}
