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

pub struct ChownApplet;
impl Applet for ChownApplet {
    fn name(&self) -> &'static str {
        "chown"
    }
    fn description(&self) -> &'static str {
        "Change file owner and group"
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
                            eprintln!("chown: invalid option -- '{}'", c as char);
                            return Ok(1);
                        }
                    }
                }
            } else {
                pos.push(a);
            }
        }
        if pos.is_empty() {
            eprintln!("chown: missing operand");
            return Ok(1);
        }
        if pos.len() < 2 {
            eprintln!(
                "chown: missing file operand after '{}'",
                pos[0].to_string_lossy()
            );
            return Ok(1);
        }
        let (u, g) = split_user_group(pos[0].as_bytes());
        if u.is_empty() && g.map(|x| x.is_empty()).unwrap_or(true) {
            eprintln!("chown: invalid owner");
            return Ok(1);
        }
        let uid = if u.is_empty() {
            u32::MAX
        } else {
            match resolve_uid(u) {
                Some(v) => v,
                None => {
                    eprintln!("chown: invalid user '{}'", String::from_utf8_lossy(u));
                    return Ok(1);
                }
            }
        };
        let gid = match g {
            Some(gr) if !gr.is_empty() => match resolve_gid(gr) {
                Some(v) => v,
                None => {
                    eprintln!("chown: invalid group '{}'", String::from_utf8_lossy(gr));
                    return Ok(1);
                }
            },
            _ => u32::MAX,
        };
        let mut rc = 0;
        for f in &pos[1..] {
            let p = Path::new(f);
            let r = if recursive {
                chown_tree(p, uid, gid, noderef)
            } else {
                match do_chown(p, uid, gid, noderef) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("chown: {}: {}", p.display(), e);
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
