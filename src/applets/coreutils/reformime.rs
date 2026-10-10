use super::common::*;
use crate::core::Result;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

applet!(
    ReformimeApplet,
    "reformime",
    "Parse MIME-encoded message",
    run_reformime
);
fn run_reformime(args: &[OsString]) -> Result<i32> {
    let (mut idx, mut want_hdr, mut outfile): (Option<usize>, bool, Option<OsString>) =
        (None, false, None);
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-i" {
            i += 1;
            if i >= args.len() {
                eprintln!("reformime: -i needs an argument");
                return Ok(1);
            }
            match lossy(&args[i]).parse() {
                Ok(n) => idx = Some(n),
                Err(_) => {
                    eprintln!("reformime: invalid index");
                    return Ok(1);
                }
            }
        } else if b.starts_with(b"-i") && b.len() > 2 {
            match String::from_utf8_lossy(&b[2..]).parse() {
                Ok(n) => idx = Some(n),
                Err(_) => {
                    eprintln!("reformime: invalid index");
                    return Ok(1);
                }
            }
        } else if b == b"-e" {
            want_hdr = true;
        } else if b == b"-o" {
            i += 1;
            if i >= args.len() {
                eprintln!("reformime: -o needs an argument");
                return Ok(1);
            }
            outfile = Some(args[i].clone());
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("reformime: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }
    let data = if files.is_empty() {
        match read_all(OsStr::from_bytes(b"-")) {
            Ok(v) => v,
            Err(_) => return Ok(1),
        }
    } else {
        match read_all(files[0].as_os_str()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("reformime: can't open '{}': {}", lossy(&files[0]), e);
                return Ok(1);
            }
        }
    };

    let text = String::from_utf8_lossy(&data);
    let mut boundary: Option<String> = None;
    for line in text.lines() {
        let l = line.trim().to_ascii_lowercase();
        if let Some(p) = l.find("boundary=") {
            let mut v = line[p + 9..].trim().trim_matches('"').trim().to_string();
            v = v.trim_matches('"').to_string();
            if !v.is_empty() {
                boundary = Some(v);
            }
        }
    }
    let mut sections: Vec<(String, Vec<u8>)> = Vec::new();
    if let Some(bnd) = boundary {
        let mark = format!("--{}", bnd);
        for part in text.split(&mark) {
            if part.contains("--") && part.trim_start().starts_with("--") {
                continue;
            }
            let (hdr, body) = match part.find("\n\n") {
                Some(p) => (part[..p].to_string(), part.as_bytes()[p + 2..].to_vec()),
                None => (String::new(), part.as_bytes().to_vec()),
            };
            if body
                .iter()
                .any(|&c| c != b'\r' && c != b'\n' && c != b' ' && c != b'\t' && c != b'-')
            {
                sections.push((hdr, body));
            }
        }
    } else {
        let (hdr, body) = match text.find("\n\n") {
            Some(p) => (text[..p].to_string(), text[p + 2..].as_bytes().to_vec()),
            None => (String::new(), data.clone()),
        };
        sections.push((hdr, body));
    }

    let mut out = wlock();
    let mut show = |n: usize| -> Result<i32> {
        if n >= sections.len() {
            eprintln!("reformime: no section {}", n);
            return Ok(1);
        }
        let (hdr, body) = &sections[n];
        let is_b64 = hdr.to_ascii_lowercase().contains("base64");
        let payload = if is_b64 {
            match b64_dec(body, true) {
                Ok(v) => v,
                Err(_) => body.clone(),
            }
        } else {
            body.clone()
        };
        if want_hdr {
            let _ = out.write_all(hdr.as_bytes());
            let _ = out.write_all(b"\n\n");
        }
        if let Some(o) = &outfile {
            match File::create(Path::new(o)) {
                Ok(mut f) => {
                    if f.write_all(&payload).is_err() {
                        return Ok(1);
                    }
                }
                Err(e) => {
                    eprintln!("reformime: can't write '{}': {}", lossy(o), e);
                    return Ok(1);
                }
            }
        } else {
            let _ = out.write_all(&payload);
        }
        Ok(0)
    };
    match idx {
        Some(n) => show(n),
        None => {
            for (n, (hdr, _)) in sections.iter().enumerate() {
                let ct = hdr
                    .lines()
                    .find(|l| l.to_ascii_lowercase().starts_with("content-type"))
                    .unwrap_or("content-type: text/plain");
                let _ = writeln!(out, "section {}: {}", n, ct);
            }
            Ok(0)
        }
    }
}

