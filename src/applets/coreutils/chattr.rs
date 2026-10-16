use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

applet!(ChattrApplet, "chattr", "Change file attributes", run_chattr);
fn run_chattr(args: &[OsString]) -> Result<i32> {
    let (mut op_arg, mut recurse) = (None::<(u8, Vec<u8>)>, false);
    let mut files: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b == b"-R" {
            recurse = true;
        } else if b == b"-V" || b == b"-f" {
        } else if b.len() > 1 && (b[1] == b'+' || b[1] == b'-' || b[1] == b'=') && b[0] == b'-' {
            eprintln!("chattr: invalid option '{}'", lossy(a));
            return Ok(1);
        } else if !b.is_empty() && (b[0] == b'+' || b[0] == b'-' || b[0] == b'=') {
            if op_arg.is_some() {
                eprintln!("chattr: only one operator allowed");
                return Ok(1);
            }
            let mut letters = Vec::new();
            for &c in &b[1..] {
                if bit_of(c).is_none() {
                    eprintln!("chattr: invalid attribute '{}'", c as char);
                    return Ok(1);
                }
                letters.push(c);
            }
            op_arg = Some((b[0], letters));
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("chattr: invalid option '{}'", lossy(a));
            return Ok(1);
        } else {
            files.push(a.clone());
        }
    }
    let (op, letters) = match op_arg {
        Some(x) => x,
        None => {
            eprintln!("chattr: needs an operator (+/-/=) argument");
            return Ok(1);
        }
    };
    if files.is_empty() {
        eprintln!("chattr: needs a file argument");
        return Ok(1);
    }

    let mut targets = files.clone();
    if recurse {
        let mut extra = Vec::new();
        for f in &files {
            if let Ok(rd) = std::fs::read_dir(Path::new(f)) {
                for e in rd.flatten() {
                    extra.push(e.path().into_os_string());
                }
            }
        }
        targets.extend(extra);
    }
    let mut rc = 0;
    for t in &targets {
        let cur = match get_flags(t.as_os_str()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("chattr: can't read flags of '{}': {}", lossy(t), e);
                rc = 1;
                continue;
            }
        };
        let mut mask = 0;
        for &c in &letters {
            mask |= bit_of(c).unwrap_or(0);
        }
        let new = match op {
            b'+' => cur | mask,
            b'-' => cur & !mask,
            _ => mask,
        };
        if let Err(e) = set_flags(t.as_os_str(), new) {
            eprintln!("chattr: can't set flags of '{}': {}", lossy(t), e);
            rc = 1;
        }
    }
    Ok(rc)
}

fn xenc(data: &[u8], fmt: &str) -> String {
    match fmt {
        "hex" => {
            let mut s = String::from("0x");
            for &b in data {
                use std::fmt::Write as _;
                let _ = write!(s, "{:02x}", b);
            }
            s
        }
        "base64" => String::from_utf8(b64_enc(data)).unwrap_or_default(),
        _ => String::from_utf8_lossy(data).into_owned(),
    }
}
