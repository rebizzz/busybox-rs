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

pub struct SeqApplet;
impl Applet for SeqApplet {
    fn name(&self) -> &'static str {
        "seq"
    }
    fn description(&self) -> &'static str {
        "Print numbers from FIRST to LAST"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sep = "\n".to_string();
        let mut pad = false;
        let mut pos_args = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-w" {
                pad = true;
            } else if bytes == b"-s" {
                if i + 1 < args.len() {
                    sep = args[i + 1].to_string_lossy().to_string();
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-s") {
                sep = arg.to_string_lossy()[2..].to_string();
            } else {
                pos_args.push(arg.to_string_lossy().to_string());
            }
            i += 1;
        }

        if pos_args.is_empty() || pos_args.len() > 3 {
            eprintln!("seq: invalid arguments");
            return Ok(1);
        }

        let (first_str, step_str, last_str) = match pos_args.len() {
            1 => ("1".to_string(), "1".to_string(), pos_args[0].clone()),
            2 => (pos_args[0].clone(), "1".to_string(), pos_args[1].clone()),
            3 => (
                pos_args[0].clone(),
                pos_args[1].clone(),
                pos_args[2].clone(),
            ),
            _ => unreachable!(),
        };

        let first: f64 = match first_str.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("seq: invalid number: {}", first_str);
                return Ok(1);
            }
        };
        let step: f64 = match step_str.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("seq: invalid number: {}", step_str);
                return Ok(1);
            }
        };
        let last: f64 = match last_str.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("seq: invalid number: {}", last_str);
                return Ok(1);
            }
        };

        if step == 0.0 {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            loop {
                if writeln!(handle, "{}", first_str).is_err() {
                    break;
                }
            }
            return Ok(0);
        }

        let argv_strs: Vec<&str> = match pos_args.len() {
            1 => vec![&pos_args[0]],
            2 => vec![&pos_args[0], &pos_args[1]],
            3 => vec![&pos_args[0], &pos_args[1], &pos_args[2]],
            _ => unreachable!(),
        };

        let mut width = 0usize;
        let mut frac_part = 0usize;
        for (idx, arg_str) in argv_strs.iter().enumerate() {
            let dot_pos = arg_str.find('.').unwrap_or(arg_str.len());
            let w = dot_pos;
            let f = arg_str.len() - dot_pos;
            if width < w {
                width = w;
            }
            if idx + 1 == argv_strs.len() {
                break;
            }
            if frac_part < f {
                frac_part = f;
            }
        }
        if frac_part > 0 {
            frac_part -= 1;
            if frac_part > 0 {
                width += frac_part + 1;
            }
        }
        if !pad {
            width = 0;
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let mut v = first;
        let mut n = 0;
        let mut sep_cur = "";

        while if step >= 0.0 { v <= last } else { v >= last } {
            let formatted = if frac_part > 0 {
                format!("{:.*}", frac_part, v)
            } else {
                format!("{:.0}", v)
            };
            let (is_neg, num_digits) = if let Some(stripped) = formatted.strip_prefix('-') {
                (true, stripped)
            } else {
                (false, formatted.as_str())
            };
            handle.write_all(sep_cur.as_bytes())?;
            if width > 0 && formatted.len() < width {
                let pad_count = width - formatted.len();
                if is_neg {
                    handle.write_all(b"-")?;
                }
                for _ in 0..pad_count {
                    handle.write_all(b"0")?;
                }
                handle.write_all(num_digits.as_bytes())?;
            } else {
                handle.write_all(formatted.as_bytes())?;
            }
            sep_cur = &sep;
            n += 1;
            v = first + (n as f64) * step;
        }

        if n > 0 {
            handle.write_all(b"\n")?;
        }
        Ok(0)
    }
}
