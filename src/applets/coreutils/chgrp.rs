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

pub struct ChgrpApplet;
impl Applet for ChgrpApplet {
    fn name(&self) -> &'static str {
        "chgrp"
    }
    fn description(&self) -> &'static str {
        "Change file group"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut recursive = false;
        let mut noderef = false;
        let mut pos: Vec<&OsString> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
                for &c in &b[1..] {
                    match c {
                        b'R' => recursive = true,
                        b'h' => noderef = true,
                        _ => {
                            eprintln!("chgrp: invalid option -- '{}'", c as char);
                            return Ok(1);
                        }
                    }
                }
            } else {
                pos.push(a);
            }
        }
        if pos.len() < 2 {
            eprintln!("chgrp: missing operand");
            return Ok(1);
        }
        let gid = match resolve_gid(pos[0].as_bytes()) {
            Some(v) => v,
            None => {
                eprintln!("chgrp: invalid group '{}'", pos[0].to_string_lossy());
                return Ok(1);
            }
        };
        let mut rc = 0;
        for f in &pos[1..] {
            let p = Path::new(f);
            let r = if recursive {
                chown_tree(p, u32::MAX, gid, noderef)
            } else {
                match do_chown(p, u32::MAX, gid, noderef) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("chgrp: {}: {}", p.display(), e);
                        1
                    }
                }
            };
            if r != 0 {
                rc = 1;
            }
        }
        Ok(rc)
    }
}
