use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;

fn b64_enc(d: &[u8]) -> Vec<u8> {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = Vec::with_capacity(d.len().div_ceil(3) * 4);
    for c in d.chunks(3) {
        let n = (c[0] as u32) << 16
            | (c.get(1).copied().unwrap_or(0) as u32) << 8
            | (c.get(2).copied().unwrap_or(0) as u32);
        o.push(T[(n >> 18) as usize & 63]);
        o.push(T[(n >> 12) as usize & 63]);
        o.push(if c.len() > 1 {
            T[(n >> 6) as usize & 63]
        } else {
            b'='
        });
        o.push(if c.len() > 2 {
            T[n as usize & 63]
        } else {
            b'='
        });
    }
    o
}
fn b64_val(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        b'=' => Some(64),
        _ => None,
    }
}
fn b64_dec(d: &[u8], ign: bool) -> std::result::Result<Vec<u8>, String> {
    let mut digs: Vec<u8> = Vec::new();
    for &c in d {
        if c == b'\n' || c == b'\r' || c == b' ' || c == b'\t' {
            continue;
        }
        match b64_val(c) {
            Some(v) => digs.push(v),
            None => {
                if ign {
                    continue;
                }
                return Err("invalid input".to_string());
            }
        }
    }
    if !digs.len().is_multiple_of(4) {
        return Err("invalid input".to_string());
    }
    let mut o = Vec::with_capacity(digs.len() / 4 * 3);
    for c in digs.chunks(4) {
        let pad = c.iter().rev().take_while(|&&v| v == 64).count();
        if pad > 2 {
            return Err("invalid input".to_string());
        }
        let mut n = 0u32;
        for (i, &v) in c.iter().enumerate() {
            let v = if v == 64 { 0 } else { v as u32 };
            n |= v << (18 - 6 * i);
        }
        o.push((n >> 16) as u8);
        if pad < 2 {
            o.push((n >> 8) as u8);
        }
        if pad < 1 {
            o.push(n as u8);
        }
    }
    Ok(o)
}
fn b32_enc(d: &[u8]) -> Vec<u8> {
    const T: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut o = Vec::with_capacity(d.len().div_ceil(5) * 8);
    for c in d.chunks(5) {
        let mut n = 0u64;
        for &b in c {
            n = (n << 8) | b as u64;
        }
        n <<= (5 - c.len()) * 8;
        let nd = (c.len() * 8).div_ceil(5);
        for i in 0..8 {
            o.push(if i < nd {
                T[(n >> (35 - 5 * i)) as usize & 31]
            } else {
                b'='
            });
        }
    }
    o
}
fn b32_dec(d: &[u8], ign: bool) -> std::result::Result<Vec<u8>, String> {
    let mut digs: Vec<u8> = Vec::new();
    for &c in d {
        if c == b'\n' || c == b'\r' || c == b' ' || c == b'\t' {
            continue;
        }
        let v = match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26 - 24 + 24 - 24),
            b'2'..=b'7' => Some(c - b'2' + 26),
            b'=' => Some(32),
            _ => None,
        };

        let v = match c {
            b'a'..=b'z' => Some(c - b'a'),
            _ => v,
        };
        match v {
            Some(v) => digs.push(v),
            None => {
                if ign {
                    continue;
                }
                return Err("invalid input".to_string());
            }
        }
    }
    if !digs.len().is_multiple_of(8) {
        return Err("invalid input".to_string());
    }
    let mut o = Vec::new();
    for c in digs.chunks(8) {
        let pad = c.iter().rev().take_while(|&&v| v == 32).count();
        let mut n = 0u64;
        for (i, &v) in c.iter().enumerate() {
            let v = if v == 32 { 0 } else { v as u64 };
            n |= v << (35 - 5 * i);
        }
        let nb = 5 - pad * 5 / 8;
        for i in 0..nb {
            o.push((n >> (32 - 8 * i)) as u8);
        }
    }
    Ok(o)
}
pub fn bxx_main(name: &str, args: &[OsString], is32: bool) -> Result<i32> {
    let (mut dec, mut ign, mut wrap) = (false, false, 76usize);
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-d" || b == b"--decode" {
            dec = true;
        } else if b == b"-i" || b == b"--ignore-garbage" {
            ign = true;
        } else if b == b"-w" {
            i += 1;
            if i >= args.len() {
                eprintln!("{}: -w needs an argument", name);
                return Ok(1);
            }
            wrap = lossy(&args[i]).parse().unwrap_or(76);
        } else if b.starts_with(b"-w") && b.len() > 2 {
            wrap = String::from_utf8_lossy(&b[2..]).parse().unwrap_or(76);
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" && !b.starts_with(b"--") {
            let mut ok = true;
            for &c in &b[1..] {
                match c {
                    b'd' => dec = true,
                    b'i' => ign = true,
                    _ => {
                        ok = false;
                        break;
                    }
                }
            }
            if !ok {
                eprintln!("{}: invalid option '{}'", name, lossy(&args[i]));
                return Ok(1);
            }
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }
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
        if dec {
            let r = if is32 {
                b32_dec(&data, ign)
            } else {
                b64_dec(&data, ign)
            };
            match r {
                Ok(v) => {
                    if out.write_all(&v).is_err() {
                        rc = 1;
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", name, e);
                    rc = 1;
                }
            }
        } else {
            let e = if is32 { b32_enc(&data) } else { b64_enc(&data) };
            if wrap == 0 {
                let _ = out.write_all(&e);
                let _ = out.write_all(b"\n");
            } else {
                for c in e.chunks(wrap) {
                    let _ = out.write_all(c);
                    let _ = out.write_all(b"\n");
                }
            }
        }
    }
    Ok(rc)
}
applet!(
    Base32Applet,
    "base32",
    "Base32 encode or decode",
    run_base32
);
fn run_base32(args: &[OsString]) -> Result<i32> {
    bxx_main("base32", args, true)
}
