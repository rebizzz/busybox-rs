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

pub struct SleepApplet;
impl Applet for SleepApplet {
    fn name(&self) -> &'static str {
        "sleep"
    }
    fn description(&self) -> &'static str {
        "Delay for a specified amount of time"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("sleep: missing operand");
            return Ok(1);
        }
        let mut total_secs: f64 = 0.0;
        for arg in args {
            let s = arg.to_string_lossy();
            let (val_str, unit) = if let Some(stripped) = s.strip_suffix('s') {
                (stripped, 1.0)
            } else if let Some(stripped) = s.strip_suffix('m') {
                (stripped, 60.0)
            } else if let Some(stripped) = s.strip_suffix('h') {
                (stripped, 3600.0)
            } else if let Some(stripped) = s.strip_suffix('d') {
                (stripped, 86400.0)
            } else {
                (s.as_ref(), 1.0)
            };
            match val_str.parse::<f64>() {
                Ok(v) => total_secs += v * unit,
                Err(_) => {
                    eprintln!("sleep: invalid number '{}'", s);
                    return Ok(1);
                }
            }
        }
        thread::sleep(Duration::from_secs_f64(total_secs));
        Ok(0)
    }
}
