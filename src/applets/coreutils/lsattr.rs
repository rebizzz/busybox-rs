use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;

applet!(LsattrApplet, "lsattr", "List file attributes", run_lsattr);
fn run_lsattr(args: &[OsString]) -> Result<i32> {
    let mut files: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b == b"--" {
            continue;
        }
        if b.len() > 1 && b[0] == b'-' && b != b"-" {
            let mut ok = true;
            for &c in &b[1..] {
                if !matches!(c, b'a' | b'd' | b'R' | b'v') {
                    ok = false;
                    break;
                }
            }
            if !ok {
                eprintln!("lsattr: invalid option '{}'", lossy(a));
                return Ok(1);
            }
            continue;
        }
        files.push(a.clone());
    }
    if files.is_empty() {
        eprintln!("lsattr: needs a file argument");
        return Ok(1);
    }
    let mut rc = 0;
    let mut out = wlock();
    for f in &files {
        match get_flags(f.as_os_str()) {
            Ok(fl) => {
                let _ = writeln!(out, "{} {}", flags_str(fl), lossy(f));
            }
            Err(e) => {
                eprintln!("lsattr: can't read flags of '{}': {}", lossy(f), e);
                rc = 1;
            }
        }
    }
    Ok(rc)
}
