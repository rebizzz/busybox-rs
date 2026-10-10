use super::common::*;
use crate::core::Result;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

applet!(
    MountpointApplet,
    "mountpoint",
    "Check if directory is a mountpoint",
    run_mountpoint
);
fn run_mountpoint(args: &[OsString]) -> Result<i32> {
    let (mut quiet, mut print_dev) = (false, false);
    let mut dirs: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b == b"-q" || b == b"--quiet" {
            quiet = true;
        } else if b == b"-d" || b == b"-x" {
            print_dev = true;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("mountpoint: invalid option '{}'", lossy(a));
            return Ok(1);
        } else {
            dirs.push(a.clone());
        }
    }
    if dirs.len() != 1 {
        eprintln!("mountpoint: needs exactly one directory");
        return Ok(1);
    }
    let p = Path::new(&dirs[0]);
    let st = match std::fs::metadata(p) {
        Ok(_) => unsafe {
            let mut s: libc::stat = std::mem::zeroed();
            let c = std::ffi::CString::new(p.as_os_str().as_bytes())
                .unwrap_or_else(|_| std::ffi::CString::new("/").unwrap());
            if libc::stat(c.as_ptr(), &mut s) != 0 {
                eprintln!("mountpoint: can't stat '{}'", lossy(&dirs[0]));
                return Ok(1);
            }
            s
        },
        Err(e) => {
            eprintln!("mountpoint: can't stat '{}': {}", lossy(&dirs[0]), e);
            return Ok(1);
        }
    };

    let canon = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let mut is_mp = false;
    if let Ok(m) = std::fs::read("/proc/mounts") {
        for line in m.split(|&c| c == b'\n') {
            let f: Vec<&[u8]> = line.split(|&c| c == b' ').collect();
            if f.len() >= 2 {
                let mp = Path::new(OsStr::from_bytes(f[1]));
                let mc = std::fs::canonicalize(mp).unwrap_or_else(|_| mp.to_path_buf());
                if mc == canon {
                    is_mp = true;
                    break;
                }
            }
        }
    }

    if !is_mp {
        if let Some(par) = p.parent() {
            let pp = if par.as_os_str().is_empty() {
                Path::new("/")
            } else {
                par
            };
            unsafe {
                let c = std::ffi::CString::new(pp.as_os_str().as_bytes())
                    .unwrap_or_else(|_| std::ffi::CString::new("/").unwrap());
                let mut ps: libc::stat = std::mem::zeroed();
                if libc::stat(c.as_ptr(), &mut ps) == 0 && ps.st_dev != st.st_dev {
                    is_mp = true;
                }
            }
        } else {
            is_mp = true;
        }
    }
    if print_dev {
        println!("{}:{}", libc::major(st.st_dev), libc::minor(st.st_dev));
    } else if !quiet {
        println!(
            "{} is {}a mountpoint",
            lossy(&dirs[0]),
            if is_mp { "" } else { "not " }
        );
    }
    if is_mp {
        Ok(0)
    } else {
        Ok(1)
    }
}

