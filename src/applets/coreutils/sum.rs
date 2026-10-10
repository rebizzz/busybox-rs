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

pub struct SumApplet;
impl Applet for SumApplet {
    fn name(&self) -> &'static str {
        "sum"
    }
    fn description(&self) -> &'static str {
        "Checksum and count the blocks in a file"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sysv = false;
        let mut bsd = false;
        let mut files: Vec<String> = Vec::new();

        let mut parsing_opts = true;
        for arg in args {
            let s = arg.to_string_lossy();
            if parsing_opts && s == "--" {
                parsing_opts = false;
                continue;
            }
            if parsing_opts && s.starts_with('-') && s.len() > 1 && s != "-" {
                for ch in s[1..].chars() {
                    match ch {
                        's' => {
                            sysv = true;
                        }
                        'r' => {
                            bsd = true;
                        }
                        _ => {
                            eprintln!("sum: unrecognized option '{}'", s);
                            return Ok(1);
                        }
                    }
                }
                continue;
            }
            files.push(s.into_owned());
        }

        let is_sysv = sysv && !bsd;

        let num_files = files.len();
        let (actual_files, print_name) = if files.is_empty() {
            (vec!["-".to_string()], is_sysv)
        } else {
            let p_name = num_files > 1 || is_sysv;
            (files, p_name)
        };

        let mut overall_success = true;

        for file in &actual_files {
            let name_to_print = if print_name { file.as_str() } else { "" };

            if is_sysv {
                let mut hasher = SysVSum::new();
                let res = if file == "-" {
                    let stdin = io::stdin();
                    let mut handle = stdin.lock();
                    let mut buf = [0u8; 8192];
                    loop {
                        match handle.read(&mut buf) {
                            Ok(0) => break Ok(()),
                            Ok(n) => hasher.update(&buf[..n]),
                            Err(e) => break Err(e),
                        }
                    }
                } else {
                    match File::open(Path::new(file)) {
                        Ok(mut f) => {
                            let mut buf = [0u8; 8192];
                            loop {
                                match f.read(&mut buf) {
                                    Ok(0) => break Ok(()),
                                    Ok(n) => hasher.update(&buf[..n]),
                                    Err(e) => break Err(e),
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("sum: can't open '{}': {}", file, e);
                            overall_success = false;
                            continue;
                        }
                    }
                };

                if let Err(e) = res {
                    eprintln!("sum: {}: {}", file, e);
                    overall_success = false;
                    continue;
                }

                let (sum, blocks) = hasher.finalize();
                println!("{} {} {}", sum, blocks, name_to_print);
            } else {
                let mut hasher = BsdSum::new();
                let res = if file == "-" {
                    let stdin = io::stdin();
                    let mut handle = stdin.lock();
                    let mut buf = [0u8; 8192];
                    loop {
                        match handle.read(&mut buf) {
                            Ok(0) => break Ok(()),
                            Ok(n) => hasher.update(&buf[..n]),
                            Err(e) => break Err(e),
                        }
                    }
                } else {
                    match File::open(Path::new(file)) {
                        Ok(mut f) => {
                            let mut buf = [0u8; 8192];
                            loop {
                                match f.read(&mut buf) {
                                    Ok(0) => break Ok(()),
                                    Ok(n) => hasher.update(&buf[..n]),
                                    Err(e) => break Err(e),
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("sum: can't open '{}': {}", file, e);
                            overall_success = false;
                            continue;
                        }
                    }
                };

                if let Err(e) = res {
                    eprintln!("sum: {}: {}", file, e);
                    overall_success = false;
                    continue;
                }

                let (sum, blocks) = hasher.finalize();
                println!("{:05} {:5} {}", sum, blocks, name_to_print);
            }
        }

        if overall_success {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}
