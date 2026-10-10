use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;

fn dump_lines(out: &mut impl Write, data: &[u8], base: u64, cols: usize, up: bool) {
    let mut off = base;
    let mut line = Vec::with_capacity(128);
    for c in data.chunks(cols.max(1)) {
        line.clear();
        let _ = write!(line, "{:08x}  ", off);
        for &b in c {
            if up {
                let _ = write!(line, "{:02X} ", b);
            } else {
                line.extend_from_slice(&[
                    b"0123456789abcdef"[(b >> 4) as usize],
                    b"0123456789abcdef"[(b & 15) as usize],
                    b' ',
                ]);
            }
        }

        let _ = write!(line, " |");
        for &b in c {
            line.push(if (32..127).contains(&b) { b } else { b'.' });
        }
        line.push(b'|');
        line.push(b'\n');
        let _ = out.write_all(&line);
        off += c.len() as u64;
    }
}

pub fn hd_main(name: &str, args: &[OsString], canonical_default: bool) -> Result<i32> {
    let (mut cols, mut off, mut len, mut verb) = (16usize, 0u64, u64::MAX, false);
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-v" {
            verb = true;
        } else if b == b"-C" {
        } else if b == b"-n" {
            i += 1;
            if i >= args.len() {
                eprintln!("{}: -n needs an argument", name);
                return Ok(1);
            }
            len = lossy(&args[i]).parse().unwrap_or(u64::MAX);
        } else if b.starts_with(b"-n") && b.len() > 2 {
            len = String::from_utf8_lossy(&b[2..]).parse().unwrap_or(u64::MAX);
        } else if b == b"-s" {
            i += 1;
            if i >= args.len() {
                eprintln!("{}: -s needs an argument", name);
                return Ok(1);
            }
            let s = lossy(&args[i]);
            off = if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                u64::from_str_radix(h, 16).unwrap_or(0)
            } else {
                s.parse().unwrap_or(0)
            };
        } else if b.starts_with(b"-s") && b.len() > 2 {
            let s = String::from_utf8_lossy(&b[2..]).into_owned();
            off = if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                u64::from_str_radix(h, 16).unwrap_or(0)
            } else {
                s.parse().unwrap_or(0)
            };
        } else if b == b"--" {
            files.extend_from_slice(&args[i + 1..]);
            break;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("{}: invalid option '{}'", name, lossy(&args[i]));
            return Ok(1);
        } else {
            files.push(args[i].clone());
        }
        let _ = (canonical_default, verb, cols);
        i += 1;
    }
    let _ = &mut cols;
    if files.is_empty() {
        files.push(OsString::from("-"));
    }
    let mut rc = 0;
    let mut out = wlock();
    for f in &files {
        let data = match read_all(f.as_os_str()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{}: can't open '{}': {}", name, lossy(f), e);
                rc = 1;
                continue;
            }
        };
        let start = (off as usize).min(data.len());
        let end = (start as u64 + len).min(data.len() as u64) as usize;
        dump_lines(&mut out, &data[start..end], off, 16, false);
        let _ = writeln!(out, "{:08x}", off + (end - start) as u64);
    }
    Ok(rc)
}
applet!(HdApplet, "hd", "Hexdump in canonical form", run_hd);
pub fn run_hd(args: &[OsString]) -> Result<i32> {
    hd_main("hd", args, true)
}
applet!(HexdumpApplet, "hexdump", "Dump file in hex", run_hexdump);
fn run_hexdump(args: &[OsString]) -> Result<i32> {
    hd_main("hexdump", args, true)
}
