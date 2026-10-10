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

pub struct PwdApplet;
impl Applet for PwdApplet {
    fn name(&self) -> &'static str {
        "pwd"
    }
    fn description(&self) -> &'static str {
        "Print the current working directory"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let dir = env::current_dir()?;
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(dir.as_os_str().as_bytes())?;
        handle.write_all(b"\n")?;
        Ok(0)
    }
}
