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

pub struct UniqApplet;
impl Applet for UniqApplet {
    fn name(&self) -> &'static str {
        "uniq"
    }
    fn description(&self) -> &'static str {
        "Report or omit repeated lines"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut count_flag = false;
        let mut dups_only = false;
        let mut unique_only = false;
        let mut skip_fields = 0usize;
        let mut skip_chars = 0usize;
        let mut check_chars = usize::MAX;
        let mut pos_args = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-c" {
                count_flag = true;
            } else if bytes == b"-d" {
                dups_only = true;
            } else if bytes == b"-u" {
                unique_only = true;
            } else if bytes == b"-f" {
                if i + 1 < args.len() {
                    skip_fields = args[i + 1].to_string_lossy().parse().unwrap_or(0);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-f") {
                skip_fields = arg.to_string_lossy()[2..].parse().unwrap_or(0);
            } else if bytes == b"-s" {
                if i + 1 < args.len() {
                    skip_chars = args[i + 1].to_string_lossy().parse().unwrap_or(0);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-s") {
                skip_chars = arg.to_string_lossy()[2..].parse().unwrap_or(0);
            } else if bytes == b"-w" {
                if i + 1 < args.len() {
                    check_chars = args[i + 1].to_string_lossy().parse().unwrap_or(usize::MAX);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-w") {
                check_chars = arg.to_string_lossy()[2..].parse().unwrap_or(usize::MAX);
            } else {
                pos_args.push(Path::new(arg));
            }
            i += 1;
        }

        let (infile, outfile) = match pos_args.len() {
            0 => (Path::new("-"), Path::new("-")),
            1 => (pos_args[0], Path::new("-")),
            2 => (pos_args[0], pos_args[1]),
            _ => {
                eprintln!("uniq: extra operand");
                return Ok(1);
            }
        };

        let content = read_bytes_or_stdin(infile)?;
        let mut lines = Vec::new();
        let mut cur = Vec::new();
        for &b in &content {
            if b == b'\n' {
                lines.push(String::from_utf8_lossy(&cur).to_string());
                cur.clear();
            } else {
                cur.push(b);
            }
        }
        if !cur.is_empty() {
            lines.push(String::from_utf8_lossy(&cur).to_string());
        }

        let extract_key = |s: &str| -> String {
            let mut cur = s;
            for _ in 0..skip_fields {
                let trimmed = cur.trim_start_matches(|c: char| c.is_ascii_whitespace());
                if let Some(pos) = trimmed.find(|c: char| c.is_ascii_whitespace()) {
                    cur = &trimmed[pos..];
                } else {
                    cur = "";
                    break;
                }
            }
            let chars: Vec<char> = cur.chars().collect();
            chars
                .into_iter()
                .skip(skip_chars)
                .take(check_chars)
                .collect()
        };

        let mut groups: Vec<(usize, String)> = Vec::new();
        for line in lines {
            let key = extract_key(&line);
            if let Some((cnt, last_line)) = groups.last_mut() {
                if extract_key(last_line) == key {
                    *cnt += 1;
                    continue;
                }
            }
            groups.push((1, line));
        }

        let mut out: Box<dyn Write> = if outfile.as_os_str() == "-" {
            Box::new(io::stdout())
        } else {
            Box::new(std::fs::File::create(outfile)?)
        };

        for (cnt, line) in groups {
            if dups_only && cnt == 1 {
                continue;
            }
            if unique_only && cnt > 1 {
                continue;
            }
            if count_flag {
                writeln!(out, "{:7} {}", cnt, line)?;
            } else {
                writeln!(out, "{}", line)?;
            }
        }
        Ok(0)
    }
}
