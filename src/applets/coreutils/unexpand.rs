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

pub struct UnexpandApplet;
impl Applet for UnexpandApplet {
    fn name(&self) -> &'static str {
        "unexpand"
    }
    fn description(&self) -> &'static str {
        "Convert spaces to tabs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut tab_size: usize = 8;
        let mut opt_all = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-a" || bytes == b"--all" {
                opt_all = true;
            } else if bytes == b"-f" || bytes == b"--first-only" {
                opt_all = false;
            } else if bytes == b"-t" || bytes == b"--tabs" {
                if i + 1 < args.len() {
                    tab_size = args[i + 1].to_string_lossy().parse().unwrap_or(8);
                    opt_all = true;
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-t") {
                tab_size = arg.to_string_lossy()[2..].parse().unwrap_or(8);
                opt_all = true;
            } else if bytes.starts_with(b"-") && bytes.len() > 1 {
                for &b in &bytes[1..] {
                    if b == b'a' {
                        opt_all = true;
                    } else if b == b'f' {
                        opt_all = false;
                    }
                }
            } else {
                files.push(Path::new(arg));
            }
            i += 1;
        }

        let has_f = args.iter().any(|a| {
            let b = a.as_bytes();
            b == b"-f" || b == b"--first-only"
        });
        if has_f {
            opt_all = false;
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for file in files {
            let content = read_bytes_or_stdin(file)?;

            let mut ptr_line_start = 0;
            while ptr_line_start < content.len() {
                let mut line_end = ptr_line_start;
                while line_end < content.len() && content[line_end] != b'\n' {
                    line_end += 1;
                }
                let has_newline = line_end < content.len() && content[line_end] == b'\n';
                let line = &content[ptr_line_start..line_end];
                ptr_line_start = if has_newline { line_end + 1 } else { line_end };

                let mut ptr = 0;
                let mut column = 0;

                while ptr < line.len() {
                    let mut len = 0;
                    while ptr < line.len() && line[ptr] == b' ' {
                        ptr += 1;
                        len += 1;
                    }
                    column += len;

                    if ptr < line.len() && line[ptr] == b'\t' {
                        column += tab_size - (column % tab_size);
                        ptr += 1;
                        continue;
                    }

                    let n = column / tab_size;
                    if n > 0 {
                        len = column % tab_size;
                        column = len;
                        for _ in 0..n {
                            handle.write_all(b"\t")?;
                        }
                    }

                    if !opt_all && ptr != 0 {
                        for _ in 0..len {
                            handle.write_all(b" ")?;
                        }
                        handle.write_all(&line[ptr..])?;
                        break;
                    }

                    let mut nspan = 0;
                    while ptr + nspan < line.len()
                        && line[ptr + nspan] != b' '
                        && line[ptr + nspan] != b'\t'
                    {
                        nspan += 1;
                    }

                    for _ in 0..len {
                        handle.write_all(b" ")?;
                    }
                    handle.write_all(&line[ptr..ptr + nspan])?;

                    let char_width = unicode_strwidth(&line[ptr..ptr + nspan]);
                    ptr += nspan;
                    column = (column + char_width) % tab_size;
                }

                if has_newline {
                    handle.write_all(b"\n")?;
                }
            }
        }
        Ok(0)
    }
}
