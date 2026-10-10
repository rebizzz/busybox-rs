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

pub struct PrintenvApplet;
impl Applet for PrintenvApplet {
    fn name(&self) -> &'static str {
        "printenv"
    }
    fn description(&self) -> &'static str {
        "Print all or part of environment"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        if args.is_empty() {
            for (k, v) in env::vars_os() {
                handle.write_all(k.as_bytes())?;
                handle.write_all(b"=")?;
                handle.write_all(v.as_bytes())?;
                handle.write_all(b"\n")?;
            }
            Ok(0)
        } else {
            let mut ret = 0;
            for arg in args {
                match env::var_os(arg) {
                    Some(val) => {
                        handle.write_all(val.as_bytes())?;
                        handle.write_all(b"\n")?;
                    }
                    None => ret = 1,
                }
            }
            Ok(ret)
        }
    }
}
