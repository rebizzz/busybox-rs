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

pub struct TeeApplet;
impl Applet for TeeApplet {
    fn name(&self) -> &'static str {
        "tee"
    }
    fn description(&self) -> &'static str {
        "Copy standard input to each FILE and standard output"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut append = false;
        let mut files = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes == b"-a" {
                append = true;
            } else if bytes == b"-i" {
            } else if !bytes.starts_with(b"-") {
                files.push(Path::new(arg));
            }
        }

        let mut handles = Vec::new();
        for f in &files {
            let file = if append {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(f)?
            } else {
                std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(f)?
            };
            handles.push(file);
        }

        let stdin = io::stdin();
        let mut stdin_handle = stdin.lock();
        let stdout = io::stdout();
        let mut stdout_handle = stdout.lock();

        let mut buf = [0u8; 8192];
        loop {
            let n = match stdin_handle.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => return Err(e.into()),
            };
            stdout_handle.write_all(&buf[..n])?;
            for h in &mut handles {
                h.write_all(&buf[..n])?;
            }
        }
        Ok(0)
    }
}
