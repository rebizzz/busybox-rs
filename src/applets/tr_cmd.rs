use std::ffi::OsString;
use std::io::{self, BufRead, Read, Write};
use std::os::unix::ffi::OsStrExt;
use crate::core::{Applet, Result};

pub struct TrApplet;
impl Applet for TrApplet {
    fn name(&self) -> &'static str { "tr" }
    fn description(&self) -> &'static str { "Translate, squeeze, and/or delete characters" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut delete_mode = false;
        let mut complement = false;
        let mut squeeze = false;
        let mut sets = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 && !bytes.starts_with(b"--") {
                for &c in &bytes[1..] {
                    match c {
                        b'd' => delete_mode = true,
                        b'c' | b'C' => complement = true,
                        b's' => squeeze = true,
                        _ => {}
                    }
                }
            } else {
                sets.push(arg.to_string_lossy().to_string());
            }
        }

        if sets.is_empty() {
            eprintln!("tr: missing operand");
            return Ok(1);
        }

        let set1 = parse_tr_set(&sets[0]);
        let set2 = if sets.len() > 1 { parse_tr_set(&sets[1]) } else { Vec::new() };

        let stdin = io::stdin();
        let mut stdin_handle = stdin.lock();
        let stdout = io::stdout();
        let mut stdout_handle = stdout.lock();

        let mut map = [0u8; 256];
        let mut in_set1 = [false; 256];
        for &b in &set1 { in_set1[b as usize] = true; }

        if complement {
            for i in 0..256 { in_set1[i] = !in_set1[i]; }
        }

        if !delete_mode {
            for i in 0..256 { map[i] = i as u8; }
            if complement {
                let last_b = set2.last().copied().unwrap_or(0);
                let mut s2_idx = 0;
                for i in 0..256 {
                    if in_set1[i] {
                        let target = if s2_idx < set2.len() {
                            let b = set2[s2_idx];
                            s2_idx += 1;
                            b
                        } else {
                            last_b
                        };
                        map[i] = target;
                    }
                }
            } else {
                let last_b = set2.last().copied().unwrap_or(0);
                for (idx, &b) in set1.iter().enumerate() {
                    let target = if idx < set2.len() { set2[idx] } else { last_b };
                    map[b as usize] = target;
                }
            }
        }

        let mut buf = [0u8; 8192];
        let mut out_buf = Vec::new();
        loop {
            let n = match stdin_handle.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => return Ok(1),
            };

            for &b in &buf[..n] {
                if delete_mode {
                    if !in_set1[b as usize] { out_buf.push(b); }
                } else {
                    out_buf.push(map[b as usize]);
                }
            }
            stdout_handle.write_all(&out_buf)?;
            out_buf.clear();
        }
        Ok(0)
    }
}

pub fn parse_tr_set(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            if s[i..].starts_with("[:digit:]") {
                for b in b'0'..=b'9' { out.push(b); }
                i += 9;
                continue;
            } else if s[i..].starts_with("[:lower:]") {
                for b in b'a'..=b'z' { out.push(b); }
                i += 9;
                continue;
            } else if s[i..].starts_with("[:upper:]") {
                for b in b'A'..=b'Z' { out.push(b); }
                i += 9;
                continue;
            } else if s[i..].starts_with("[:alpha:]") {
                for b in b'A'..=b'Z' { out.push(b); }
                for b in b'a'..=b'z' { out.push(b); }
                i += 9;
                continue;
            } else if s[i..].starts_with("[:alnum:]") {
                // Notice BusyBox order: 0-9, then A-Z, then a-z
                for b in b'0'..=b'9' { out.push(b); }
                for b in b'A'..=b'Z' { out.push(b); }
                for b in b'a'..=b'z' { out.push(b); }
                i += 9;
                continue;
            } else if s[i..].starts_with("[:space:]") {
                // Busybox order: \t, \n, \v, \f, \r, ' '
                out.push(b'\t');
                out.push(b'\n');
                out.push(0x0b);
                out.push(0x0c);
                out.push(b'\r');
                out.push(b' ');
                i += 9;
                continue;
            } else if s[i..].starts_with("[:blank:]") {
                // Busybox order: \t, ' '
                out.push(b'\t');
                out.push(b' ');
                i += 9;
                continue;
            } else if s[i..].starts_with("[:xdigit:]") {
                for b in b'0'..=b'9' { out.push(b); }
                for b in b'A'..=b'F' {
                    out.push(b);
                }
                for b in b'a'..=b'f' {
                    out.push(b);
                }
                i += 10;
                continue;
            } else if s[i..].starts_with("[:punct:]") {
                for b in 0u8..=127u8 {
                    if (b >= 33 && b <= 126) && !b.is_ascii_alphanumeric() && !b.is_ascii_whitespace() {
                        out.push(b);
                    }
                }
                i += 9;
                continue;
            } else if s[i..].starts_with("[:cntrl:]") {
                for b in 0u8..=31u8 { out.push(b); }
                out.push(127);
                i += 9;
                continue;
            } else if s[i..].starts_with("[=") && s[i..].len() >= 5 && s[i..].ends_with("=]") || (s[i..].len() >= 5 && &s[i..i+2] == "[=" && &s[i+3..i+5] == "=]") {
                out.push(bytes[i + 2]);
                i += 5;
                continue;
            }
        }

        if i + 2 < bytes.len() && bytes[i + 1] == b'-' && bytes[i] <= bytes[i + 2] && bytes[i] != b'\\' {
            let start = bytes[i];
            let end = bytes[i + 2];
            for b in start..=end { out.push(b); }
            i += 3;
            continue;
        }

        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 1;
            match bytes[i] {
                b'n' => out.push(b'\n'),
                b't' => out.push(b'\t'),
                b'r' => out.push(b'\r'),
                b'\\' => out.push(b'\\'),
                b'-' => out.push(b'-'),
                other => out.push(other),
            }
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    out
}
