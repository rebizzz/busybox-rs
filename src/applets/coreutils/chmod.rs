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

pub struct ChmodApplet;
impl Applet for ChmodApplet {
    fn name(&self) -> &'static str {
        "chmod"
    }
    fn description(&self) -> &'static str {
        "Change file mode bits"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut recursive = false;
        let mut pos: Vec<&OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let a = &args[i];
            let b = a.as_bytes();
            if b == b"--" {
                for r in &args[i + 1..] {
                    pos.push(r);
                }
                break;
            } else if b.starts_with(b"-") && b.len() > 1 {
                for &c in &b[1..] {
                    if c == b'R' {
                        recursive = true;
                    } else {
                        eprintln!("chmod: invalid option -- '{}'", c as char);
                        return Ok(1);
                    }
                }
            } else {
                pos.push(a);
            }
            i += 1;
        }
        if pos.len() < 2 {
            eprintln!("chmod: missing operand");
            return Ok(1);
        }
        let mode = match parse_octal(pos[0].as_bytes()) {
            Some(m) => m,
            None => {
                eprintln!("chmod: invalid mode '{}'", pos[0].to_string_lossy());
                return Ok(1);
            }
        };
        let mut rc = 0;
        for f in &pos[1..] {
            let p = Path::new(f);
            if recursive && fs::metadata(p).map(|m| m.is_dir()).unwrap_or(false) {
                if chmod_tree(p, mode) != 0 {
                    rc = 1;
                }
            } else if let Err(e) = do_chmod(p, mode) {
                eprintln!("chmod: {}: {}", p.display(), e);
                rc = 1;
            }
        }
        Ok(rc)
    }
}
