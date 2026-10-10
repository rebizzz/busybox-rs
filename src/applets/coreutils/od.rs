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

#[derive(Clone, Copy, PartialEq, Eq)]
enum OdType {
    Oct,
    Hex,
    Dec,
    Char,
    Named,
}
fn od_addr(buf: &mut Vec<u8>, off: u64, radix: u8) {
    let mut digs = [0u8; 24];
    let mut nd = 0;
    let mut v = off;
    if v == 0 {
        digs[0] = b'0';
        nd = 1;
    }
    while v > 0 {
        let d = (v % radix as u64) as u8;
        digs[nd] = if d < 10 { d + b'0' } else { d - 10 + b'a' };
        v /= radix as u64;
        nd += 1;
    }
    for _ in nd..7 {
        buf.push(b'0');
    }
    for k in (0..nd).rev() {
        buf.push(digs[k]);
    }
}
fn od_byte(buf: &mut Vec<u8>, b: u8, t: OdType, signed: bool) {
    buf.push(b' ');
    match t {
        OdType::Oct => {
            buf.push(b'0' + (b >> 6));
            buf.push(b'0' + ((b >> 3) & 7));
            buf.push(b'0' + (b & 7));
        }
        OdType::Hex => {
            for sh in [4u8, 0u8] {
                let d = (b >> sh) & 15;
                buf.push(if d < 10 { d + b'0' } else { d - 10 + b'a' });
            }
        }
        OdType::Dec => {
            let neg = signed && b >= 128;
            let m: u16 = if neg { 256 - b as u16 } else { b as u16 };
            if neg {
                buf.push(b'-');
            }
            buf.push(b'0' + (m / 100) as u8);
            buf.push(b'0' + ((m / 10) % 10) as u8);
            buf.push(b'0' + (m % 10) as u8);
        }
        OdType::Char => match b {
            0 => buf.extend_from_slice(b"  \\0"),
            7 => buf.extend_from_slice(b"  \\a"),
            8 => buf.extend_from_slice(b"  \\b"),
            9 => buf.extend_from_slice(b"  \\t"),
            10 => buf.extend_from_slice(b"  \\n"),
            11 => buf.extend_from_slice(b"  \\v"),
            12 => buf.extend_from_slice(b"  \\f"),
            13 => buf.extend_from_slice(b"  \\r"),
            32..=126 => {
                buf.extend_from_slice(b"   ");
                buf.push(b);
            }
            _ => {
                buf.push(b' ');
                buf.push(b'0' + (b >> 6));
                buf.push(b'0' + ((b >> 3) & 7));
                buf.push(b'0' + (b & 7));
            }
        },
        OdType::Named => {
            const N: [&[u8; 3]; 33] = [
                b"nul", b"soh", b"stx", b"etx", b"eot", b"enq", b"ack", b"bel", b" bs", b" ht",
                b" nl", b" vt", b" ff", b" cr", b" so", b" si", b"dle", b"dc1", b"dc2", b"dc3",
                b"dc4", b"nak", b"syn", b"etb", b"can", b" em", b"sub", b"esc", b" fs", b" gs",
                b" rs", b" us", b" sp",
            ];
            if b < 33 {
                buf.push(b' ');
                buf.extend_from_slice(&N[b as usize][..]);
            } else if b == 127 {
                buf.extend_from_slice(b" del");
            } else if b < 127 {
                buf.extend_from_slice(b"   ");
                buf.push(b);
            } else {
                buf.push(b' ');
                buf.push(b'0' + (b >> 6));
                buf.push(b'0' + ((b >> 3) & 7));
                buf.push(b'0' + (b & 7));
            }
        }
    }
}
pub struct OdApplet;
impl Applet for OdApplet {
    fn name(&self) -> &'static str {
        "od"
    }
    fn description(&self) -> &'static str {
        "Dump files in octal and other formats"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let (mut radix, mut show) = (8u8, true);
        let (mut typ, mut signed, mut verbose) = (OdType::Oct, false, false);
        let (mut skip, mut max) = (0u64, u64::MAX);
        let mut files: Vec<&Path> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for a in &args[i + 1..] {
                    files.push(Path::new(a));
                }
                break;
            }
            if b.len() > 1 && b[0] == b'-' && b != b"-" {
                let mut j = 1;
                let mut brk = false;
                while j < b.len() {
                    match b[j] {
                        b'v' => {
                            verbose = true;
                            j += 1;
                        }
                        b'x' => {
                            typ = OdType::Hex;
                            signed = false;
                            j += 1;
                        }
                        b'o' | b'B' => {
                            typ = OdType::Oct;
                            signed = false;
                            j += 1;
                        }
                        b'd' => {
                            typ = OdType::Dec;
                            signed = true;
                            j += 1;
                        }
                        b'u' => {
                            typ = OdType::Dec;
                            signed = false;
                            j += 1;
                        }
                        b'c' => {
                            typ = OdType::Char;
                            j += 1;
                        }
                        b'a' => {
                            typ = OdType::Named;
                            j += 1;
                        }
                        b'A' | b't' | b'j' | b'N' => {
                            let opt = b[j];
                            let v = take_val(b, j, &mut i, args);
                            match opt {
                                b'A' => match v.first().copied().unwrap_or(0) {
                                    b'd' => {
                                        radix = 10;
                                        show = true;
                                    }
                                    b'o' => {
                                        radix = 8;
                                        show = true;
                                    }
                                    b'x' => {
                                        radix = 16;
                                        show = true;
                                    }
                                    b'n' => show = false,
                                    _ => {
                                        eprintln!("od: bad -A");
                                        return Ok(1);
                                    }
                                },
                                b't' => match v.as_slice() {
                                    b"x1" => {
                                        typ = OdType::Hex;
                                        signed = false;
                                    }
                                    b"o1" => {
                                        typ = OdType::Oct;
                                        signed = false;
                                    }
                                    b"d1" => {
                                        typ = OdType::Dec;
                                        signed = true;
                                    }
                                    b"u1" => {
                                        typ = OdType::Dec;
                                        signed = false;
                                    }
                                    b"c" => typ = OdType::Char,
                                    b"a" => typ = OdType::Named,
                                    _ => {
                                        eprintln!("od: bad -t (x1/o1/d1/u1/c/a)");
                                        return Ok(1);
                                    }
                                },
                                b'j' | b'N' => {
                                    match String::from_utf8_lossy(&v).trim().parse::<u64>() {
                                        Ok(n) => {
                                            if opt == b'j' {
                                                skip = n;
                                            } else {
                                                max = n;
                                            }
                                        }
                                        Err(_) => {
                                            eprintln!("od: bad number");
                                            return Ok(1);
                                        }
                                    }
                                }
                                _ => {}
                            }
                            brk = true;
                            break;
                        }
                        _ => j += 1,
                    }
                }
                if brk {
                    i += 1;
                    continue;
                }
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }
        if files.is_empty() {
            files.push(Path::new("-"));
        }
        let out = io::stdout();
        let mut w = out.lock();
        let mut chunk = [0u8; 16];
        let mut row = Vec::with_capacity(128);
        let mut prev = [0u8; 16];
        let (mut plen, mut starred, mut off) = (0usize, false, 0u64);
        'fs: for f in &files {
            let mut r = open_input(f)?;
            while skip > 0 {
                let mut d = [0u8; 8192];
                let n = r.read(&mut d[..(skip as usize).min(8192)])?;
                if n == 0 {
                    break;
                }
                skip -= n as u64;
            }
            loop {
                if max == 0 {
                    break 'fs;
                }
                let want = (16usize).min(max as usize);
                let mut got = 0;
                while got < want {
                    match r.read(&mut chunk[got..want])? {
                        0 => break,
                        n => got += n,
                    }
                }
                if got == 0 {
                    break;
                }
                max -= got as u64;
                if !verbose && got == 16 && plen == 16 && chunk == prev {
                    if !starred {
                        w.write_all(b"*\n")?;
                        starred = true;
                    }
                    off += 16;
                    continue;
                }
                starred = false;
                prev[..got].copy_from_slice(&chunk[..got]);
                plen = got;
                row.clear();
                if show {
                    od_addr(&mut row, off, radix);
                }
                for &b in &chunk[..got] {
                    od_byte(&mut row, b, typ, signed);
                }
                row.push(b'\n');
                w.write_all(&row)?;
                off += got as u64;
                if got < 16 {
                    break;
                }
            }
        }
        if show {
            row.clear();
            od_addr(&mut row, off, radix);
            row.push(b'\n');
            w.write_all(&row)?;
        }
        Ok(0)
    }
}

fn convert_stream<R: BufRead, W: Write>(mut r: R, mut w: W, to_unix: bool) -> io::Result<()> {
    let mut buf = [0u8; 8192];
    let (mut pend, mut pcr) = (false, false);
    loop {
        let n = r.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let mut out = Vec::with_capacity(n + 16);
        for &b in &buf[..n] {
            if to_unix {
                if pend {
                    pend = false;
                    if b == b'\n' {
                        out.push(b'\n');
                    } else {
                        out.push(b'\r');
                        if b == b'\r' {
                            pend = true;
                        } else {
                            out.push(b);
                        }
                    }
                } else if b == b'\r' {
                    pend = true;
                } else {
                    out.push(b);
                }
            } else {
                if b == b'\n' && !pcr {
                    out.push(b'\r');
                }
                out.push(b);
                pcr = b == b'\r';
            }
        }
        w.write_all(&out)?;
    }
    if to_unix && pend {
        w.write_all(b"\r")?;
    }
    Ok(())
}
fn convert_file(path: &Path, to_unix: bool, tag: &str) -> Result<i32> {
    let tmp = path.with_extension("bb-tmp");
    let r: io::Result<()> = (|| {
        convert_stream(
            BufReader::new(std::fs::File::open(path)?),
            std::fs::File::create(&tmp)?,
            to_unix,
        )?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    })();
    if let Err(e) = r {
        let _ = std::fs::remove_file(&tmp);
        eprintln!("{tag}: {}: {e}", path.display());
        return Ok(1);
    }
    Ok(0)
}
