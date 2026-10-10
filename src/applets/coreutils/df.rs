use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::{OsStr, OsString};
use std::fs::{self};
use std::io::{self, BufRead, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct DfApplet;
impl Applet for DfApplet {
    fn name(&self) -> &'static str {
        "df"
    }
    fn description(&self) -> &'static str {
        "Report filesystem disk space usage"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut human = false;
        let mut div: u64 = 1024;
        let mut inodes = false;
        let mut pos: Vec<&Path> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
                for &c in &b[1..] {
                    match c {
                        b'h' => human = true,
                        b'k' => div = 1024,
                        b'm' => div = 1024 * 1024,
                        b'P' => {}
                        b'i' => inodes = true,
                        b'a' => {}
                        _ => {}
                    }
                }
            } else {
                pos.push(Path::new(a));
            }
        }
        let all_mounts = read_mounts();
        let mounts: Vec<(Vec<u8>, Vec<u8>)> = if pos.is_empty() {
            all_mounts
        } else {
            pos.iter()
                .map(|p| {
                    let pb = p.as_os_str().as_bytes();
                    let dev = find_dev(&all_mounts, pb).unwrap_or(b"-").to_vec();
                    (dev, pb.to_vec())
                })
                .collect()
        };
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut head = Vec::new();
        head.extend_from_slice(b"Filesystem ");
        head.extend_from_slice(if inodes {
            b"Inodes IUsed IFree IUse% "
        } else if human {
            b"Size Used Avail Use% "
        } else {
            b"1K-blocks Used Available Use% "
        });
        head.extend_from_slice(b"Mounted on\n");
        out.write_all(&head)?;
        let mut rc = 0;
        for (dev, mnt) in &mounts {
            let target = if mnt == b"-" { dev } else { mnt };
            let p = Path::new(OsStr::from_bytes(target));
            let c = match path_cstr(p) {
                Some(v) => v,
                None => {
                    rc = 1;
                    continue;
                }
            };
            let mut v: libc::statvfs = unsafe { std::mem::zeroed() };
            if unsafe { libc::statvfs(c.as_ptr(), &mut v) } != 0 {
                eprintln!("df: {}: {}", p.display(), io::Error::last_os_error());
                rc = 1;
                continue;
            }
            let mut line = Vec::new();
            line.extend_from_slice(dev);
            line.push(b' ');
            df_row(&mut line, &v, human, div, inodes);
            line.push(b' ');
            line.extend_from_slice(if mnt == b"-" { target } else { mnt });
            line.push(b'\n');
            out.write_all(&line)?;
        }
        Ok(rc)
    }
}

fn df_row(line: &mut Vec<u8>, v: &libc::statvfs, human: bool, div: u64, inodes: bool) {
    if inodes {
        put_num(line, v.f_files);
        line.push(b' ');
        put_num(line, v.f_files - v.f_ffree);
        line.push(b' ');
        put_num(line, v.f_ffree);
        line.push(b' ');
        let pct = (v.f_files - v.f_ffree)
            .saturating_mul(100)
            .checked_div(v.f_files)
            .unwrap_or(0);
        put_num(line, pct);
        line.push(b'%');
        return;
    }
    let bsize = v.f_bsize;
    let tot = v.f_blocks * bsize;
    let free = v.f_bavail * bsize;
    let used = tot.saturating_sub(v.f_bfree * bsize);
    if human {
        put_human(line, tot);
        line.push(b' ');
        put_human(line, used);
        line.push(b' ');
        put_human(line, free);
    } else {
        put_num(line, tot / div);
        line.push(b' ');
        put_num(line, used / div);
        line.push(b' ');
        put_num(line, free / div);
    }
    line.push(b' ');
    let pct = used.saturating_mul(100).checked_div(tot).unwrap_or(0);
    put_num(line, pct);
    line.push(b'%');
}

fn find_dev<'a>(mounts: &'a [(Vec<u8>, Vec<u8>)], path: &[u8]) -> Option<&'a [u8]> {
    let mut best: Option<&'a [u8]> = None;
    let mut best_len = 0;
    for (dev, mnt) in mounts {
        let hit = path == mnt
            || (path.starts_with(mnt.as_slice())
                && (mnt == b"/" || path.get(mnt.len()) == Some(&b'/')));
        if hit && mnt.len() >= best_len {
            best_len = mnt.len();
            best = Some(dev);
        }
    }
    best
}

fn read_mounts() -> Vec<(Vec<u8>, Vec<u8>)> {
    let data = fs::read("/proc/mounts").unwrap_or_default();
    let mut out = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        if line.is_empty() || line[0] == b'#' {
            continue;
        }
        let mut parts = line.split(|&b| b == b' ');
        let dev = parts.next().unwrap_or(b"").to_vec();
        let mnt = parts.next().unwrap_or(b"").to_vec();
        if dev.is_empty() || mnt.is_empty() {
            continue;
        }
        out.push((dev, mnt));
    }
    if out.is_empty() {
        out.push((b"/dev/root".to_vec(), b"/".to_vec()));
    }
    out
}
