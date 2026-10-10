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

pub struct YesApplet;
impl Applet for YesApplet {
    fn name(&self) -> &'static str {
        "yes"
    }
    fn description(&self) -> &'static str {
        "Output a string repeatedly until killed"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let msg = if args.is_empty() {
            b"y".to_vec()
        } else {
            let mut out = Vec::new();
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push(b' ');
                }
                out.extend_from_slice(a.as_bytes());
            }
            out
        };
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        loop {
            if handle.write_all(&msg).is_err() || handle.write_all(b"\n").is_err() {
                break;
            }
        }
        Ok(0)
    }
}
