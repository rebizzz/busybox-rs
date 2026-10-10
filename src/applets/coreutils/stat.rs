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

pub struct StatApplet;
impl Applet for StatApplet {
    fn name(&self) -> &'static str {
        "stat"
    }
    fn description(&self) -> &'static str {
        "Display file or filesystem status"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut fmt: Option<&[u8]> = None;
        let mut fs_mode = false;
        let mut follow = false;
        let mut terse = false;
        let mut pos: Vec<&Path> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for r in &args[i + 1..] {
                    pos.push(Path::new(r));
                }
                break;
            } else if b == b"-c" {
                i += 1;
                if i >= args.len() {
                    eprintln!("stat: option requires an argument -- 'c'");
                    return Ok(1);
                }
                fmt = Some(args[i].as_bytes());
            } else if b.starts_with(b"-c") && b.len() > 2 {
                fmt = Some(&args[i].as_bytes()[2..]);
            } else if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
                for &c in &b[1..] {
                    match c {
                        b'f' => fs_mode = true,
                        b'L' => follow = true,
                        b't' => terse = true,
                        _ => {
                            eprintln!("stat: invalid option -- '{}'", c as char);
                            return Ok(1);
                        }
                    }
                }
            } else {
                pos.push(Path::new(&args[i]));
            }
            i += 1;
        }
        if pos.is_empty() {
            eprintln!("stat: missing operand");
            return Ok(1);
        }
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut rc = 0;
        for p in pos {
            if emit_stat(&mut out, p, fmt, fs_mode, follow, terse)? != 0 {
                rc = 1;
            }
        }
        Ok(rc)
    }
}

fn emit_stat(
    out: &mut impl Write,
    p: &Path,
    fmt: Option<&[u8]>,
    fs_mode: bool,
    follow: bool,
    terse: bool,
) -> Result<i32> {
    if fs_mode {
        match fs_stat_line(p) {
            Some(line) => {
                out.write_all(&line)?;
                out.write_all(b"\n")?;
                return Ok(0);
            }
            None => {
                eprintln!("stat: cannot stat '{}'", p.display());
                return Ok(1);
            }
        }
    }
    let st = match lstat_of(p, follow) {
        Some(s) => s,
        None => {
            eprintln!("stat: cannot stat '{}'", p.display());
            return Ok(1);
        }
    };
    if let Some(f) = fmt {
        let mut line = Vec::new();
        stat_format(&mut line, f, p, &st);
        out.write_all(&line)?;
        out.write_all(b"\n")?;
    } else if terse {
        let mut line = Vec::new();
        line.extend_from_slice(p.as_os_str().as_bytes());
        line.push(b' ');
        put_num(&mut line, st.st_size as u64);
        line.push(b' ');
        put_num(&mut line, st.st_blocks as u64);
        line.push(b' ');
        put_num(&mut line, (st.st_mode & 0o7777) as u64);
        line.push(b' ');
        put_num(&mut line, st.st_uid as u64);
        line.push(b' ');
        put_num(&mut line, st.st_gid as u64);
        line.push(b' ');
        put_num(&mut line, st.st_ino);
        out.write_all(&line)?;
        out.write_all(b"\n")?;
    } else {
        let mut line = Vec::new();
        line.extend_from_slice(b"  File: ");
        line.extend_from_slice(p.as_os_str().as_bytes());
        line.extend_from_slice(b"\n  Size: ");
        put_num(&mut line, st.st_size as u64);
        line.extend_from_slice(b"  Blocks: ");
        put_num(&mut line, st.st_blocks as u64);
        line.extend_from_slice(b"\nAccess: (");
        let mut oct = Vec::new();
        put_num(&mut oct, (st.st_mode & 0o7777) as u64);
        line.extend_from_slice(&oct);
        line.extend_from_slice(b"/");
        line.push(file_type_char(st.st_mode as u32));
        put_mode_str(&mut line, st.st_mode as u32);
        line.extend_from_slice(b")  Uid: (");
        put_num(&mut line, st.st_uid as u64);
        line.extend_from_slice(b"/");
        line.extend_from_slice(&uid_name(st.st_uid));
        line.extend_from_slice(b")  Gid: (");
        put_num(&mut line, st.st_gid as u64);
        line.extend_from_slice(b"/");
        line.extend_from_slice(&gid_name(st.st_gid));
        line.extend_from_slice(b")");
        out.write_all(&line)?;
        out.write_all(b"\n")?;
    }
    Ok(0)
}

fn stat_format(buf: &mut Vec<u8>, fmt: &[u8], p: &Path, st: &libc::stat) {
    let mut i = 0;
    while i < fmt.len() {
        if fmt[i] != b'%' || i + 1 >= fmt.len() {
            buf.push(fmt[i]);
            i += 1;
            continue;
        }
        i += 1;
        match fmt[i] {
            b'a' => put_oct(buf, (st.st_mode & 0o7777) as u64),
            b'A' => {
                buf.push(file_type_char(st.st_mode));
                put_mode_str(buf, st.st_mode);
            }
            b's' => put_num(buf, st.st_size as u64),
            b'b' => put_num(buf, st.st_blocks as u64),
            b'B' => put_num(buf, 512),
            b'h' => put_num(buf, st.st_nlink),
            b'u' => put_num(buf, st.st_uid as u64),
            b'g' => put_num(buf, st.st_gid as u64),
            b'U' => buf.extend_from_slice(&uid_name(st.st_uid)),
            b'G' => buf.extend_from_slice(&gid_name(st.st_gid)),
            b'n' => buf.extend_from_slice(p.as_os_str().as_bytes()),
            b'F' => buf.extend_from_slice(match st.st_mode & libc::S_IFMT {
                libc::S_IFDIR => b"directory",
                libc::S_IFLNK => b"symbolic link",
                libc::S_IFCHR => b"character device",
                libc::S_IFBLK => b"block device",
                libc::S_IFIFO => b"fifo",
                libc::S_IFSOCK => b"socket",
                _ => b"regular file",
            }),
            b'i' => put_num(buf, st.st_ino),
            b'd' => put_num(buf, st.st_dev),
            _ => {
                buf.push(b'%');
                buf.push(fmt[i]);
            }
        }
        i += 1;
    }
}

fn fs_stat_line(p: &Path) -> Option<Vec<u8>> {
    let c = path_cstr(p)?;
    let mut v: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut v) } != 0 {
        return None;
    }
    let mut b = Vec::new();
    b.extend_from_slice(p.as_os_str().as_bytes());
    b.extend_from_slice(b" ");
    put_num(&mut b, v.f_bsize as u64);
    b.extend_from_slice(b" ");
    put_num(&mut b, v.f_blocks);
    b.extend_from_slice(b" ");
    put_num(&mut b, v.f_bavail);
    Some(b)
}

fn du_bytes(p: &Path, follow: bool, apparent: bool, one_fs: bool, top_dev: u64) -> u64 {
    let st = match lstat_of(p, follow) {
        Some(s) => s,
        None => return 0,
    };
    if one_fs && st.st_dev != top_dev {
        return 0;
    }
    let here = if apparent {
        st.st_size.max(0) as u64
    } else {
        (st.st_blocks.max(0) as u64) * 512
    };
    if (st.st_mode & libc::S_IFMT) != libc::S_IFDIR {
        return here;
    }
    let mut sum = here;
    if let Ok(rd) = fs::read_dir(p) {
        for e in rd.flatten() {
            sum += du_bytes(&e.path(), follow, apparent, one_fs, top_dev);
        }
    }
    sum
}

