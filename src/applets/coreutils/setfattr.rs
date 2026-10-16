use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

applet!(
    SetfattrApplet,
    "setfattr",
    "Set extended attributes",
    run_setfattr
);
fn run_setfattr(args: &[OsString]) -> Result<i32> {
    let (mut name, mut val, mut remove, mut no_der) =
        (None::<Vec<u8>>, None::<Vec<u8>>, false, false);
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-h" {
            no_der = true;
        } else if b == b"-n" {
            i += 1;
            if i >= args.len() {
                eprintln!("setfattr: -n needs an argument");
                return Ok(1);
            }
            name = Some(ab(&args[i]).to_vec());
        } else if b.starts_with(b"-n") && b.len() > 2 {
            name = Some(b[2..].to_vec());
        } else if b == b"-v" {
            i += 1;
            if i >= args.len() {
                eprintln!("setfattr: -v needs an argument");
                return Ok(1);
            }
            val = Some(ab(&args[i]).to_vec());
        } else if b.starts_with(b"-v") && b.len() > 2 {
            val = Some(b[2..].to_vec());
        } else if b == b"-x" {
            i += 1;
            if i >= args.len() {
                eprintln!("setfattr: -x needs an argument");
                return Ok(1);
            }
            name = Some(ab(&args[i]).to_vec());
            remove = true;
        } else if b.starts_with(b"-x") && b.len() > 2 {
            name = Some(b[2..].to_vec());
            remove = true;
        } else if b == b"--" {
            files.extend_from_slice(&args[i + 1..]);
            break;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("setfattr: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }
    let nm = match name {
        Some(n) => n,
        None => {
            eprintln!("setfattr: needs -n NAME (or -x NAME)");
            return Ok(1);
        }
    };
    if !remove && val.is_none() {
        eprintln!("setfattr: needs -v VALUE");
        return Ok(1);
    }
    if files.is_empty() {
        eprintln!("setfattr: needs a file argument");
        return Ok(1);
    }
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let cn = match CString::new(nm) {
        Ok(c) => c,
        Err(_) => return Ok(1),
    };
    let mut rc = 0;
    for f in &files {
        let cp = match CString::new(f.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => {
                rc = 1;
                continue;
            }
        };
        let r = if remove {
            if no_der {
                unsafe { libc::lremovexattr(cp.as_ptr(), cn.as_ptr()) }
            } else {
                unsafe { libc::removexattr(cp.as_ptr(), cn.as_ptr()) }
            }
        } else {
            let vv = val.clone().unwrap_or_default();
            if no_der {
                unsafe { libc::lsetxattr(cp.as_ptr(), cn.as_ptr(), vv.as_ptr() as _, vv.len(), 0) }
            } else {
                unsafe { libc::setxattr(cp.as_ptr(), cn.as_ptr(), vv.as_ptr() as _, vv.len(), 0) }
            }
        };
        if r < 0 {
            eprintln!(
                "setfattr: can't set '{}': {}",
                lossy(f),
                std::io::Error::last_os_error()
            );
            rc = 1;
        }
    }
    Ok(rc)
}

