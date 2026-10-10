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

pub struct WhoamiApplet;
impl Applet for WhoamiApplet {
    fn name(&self) -> &'static str {
        "whoami"
    }
    fn description(&self) -> &'static str {
        "Print effective user name"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        if let Some(user) = crate::core::platform::get_current_username() {
            println!("{}", user);
            Ok(0)
        } else {
            eprintln!("whoami: cannot find name for user ID");
            Ok(1)
        }
    }
}
