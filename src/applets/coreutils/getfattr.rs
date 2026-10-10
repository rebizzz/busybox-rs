use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

applet!(
    GetfattrApplet,
    "getfattr",
    "Get extended attributes",
    run_getfattr
);
fn run_getfattr(args: &[OsString]) -> Result<i32> {
    let (mut name, mut dump, mut enc, mut no_der) =
        (None::<String>, false, String::from("text"), false);
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-d" || b == b"--dump" {
            dump = true;
        } else if b == b"-h" {
            no_der = true;
        } else if b == b"-n" {
            i += 1;
            if i >= args.len() {
                eprintln!("getfattr: -n needs an argument");
                return Ok(1);
            }
            name = Some(lossy(&args[i]));
        } else if b.starts_with(b"-n") && b.len() > 2 {
            name = Some(String::from_utf8_lossy(&b[2..]).into_owned());
        } else if b == b"-e" {
            i += 1;
            if i >= args.len() {
                eprintln!("getfattr: -e needs an argument");
                return Ok(1);
            }
            enc = lossy(&args[i]);
        } else if b.starts_with(b"-e") && b.len() > 2 {
            enc = String::from_utf8_lossy(&b[2..]).into_owned();
        } else if b == b"--" {
            files.extend_from_slice(&args[i + 1..]);
            break;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("getfattr: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }
    if files.is_empty() || (name.is_none() && !dump) {
        eprintln!("getfattr: needs -n NAME or -d plus a file argument");
        return Ok(1);
    }
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let mut rc = 0;
    let mut out = wlock();
    for f in &files {
        let cp = match CString::new(f.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => {
                rc = 1;
                continue;
            }
        };
        let names: Vec<String> = if dump {
            let mut sz = unsafe { libc::listxattr(cp.as_ptr(), std::ptr::null_mut(), 0) };
            if no_der {
                sz = unsafe { libc::llistxattr(cp.as_ptr(), std::ptr::null_mut(), 0) };
            }
            if sz < 0 {
                eprintln!("getfattr: can't list '{}'", lossy(f));
                rc = 1;
                continue;
            }
            let mut buf = vec![0u8; sz as usize];
            let r = if no_der {
                unsafe { libc::llistxattr(cp.as_ptr(), buf.as_mut_ptr() as _, buf.len()) }
            } else {
                unsafe { libc::listxattr(cp.as_ptr(), buf.as_mut_ptr() as _, buf.len()) }
            };
            if r < 0 {
                eprintln!("getfattr: can't list '{}'", lossy(f));
                rc = 1;
                continue;
            }
            buf[..r as usize]
                .split(|&c| c == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .collect()
        } else {
            vec![name.clone().unwrap_or_default()]
        };
        for nm in &names {
            let cn = match CString::new(nm.as_bytes()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let mut sz =
                unsafe { libc::getxattr(cp.as_ptr(), cn.as_ptr(), std::ptr::null_mut(), 0) };
            if no_der {
                sz = unsafe { libc::lgetxattr(cp.as_ptr(), cn.as_ptr(), std::ptr::null_mut(), 0) };
            }
            if sz < 0 {
                eprintln!("getfattr: '{}' has no '{}'", lossy(f), nm);
                rc = 1;
                continue;
            }
            let mut buf = vec![0u8; sz as usize];
            let r = if no_der {
                unsafe {
                    libc::lgetxattr(cp.as_ptr(), cn.as_ptr(), buf.as_mut_ptr() as _, buf.len())
                }
            } else {
                unsafe {
                    libc::getxattr(cp.as_ptr(), cn.as_ptr(), buf.as_mut_ptr() as _, buf.len())
                }
            };
            if r < 0 {
                rc = 1;
                continue;
            }
            let _ = writeln!(out, "{}=\"{}\"", nm, xenc(&buf[..r as usize], &enc));
        }
    }
    Ok(rc)
}
