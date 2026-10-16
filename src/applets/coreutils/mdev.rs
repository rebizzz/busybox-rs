use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

applet!(MdevApplet, "mdev", "Coldplug device helper", run_mdev);
fn run_mdev(args: &[OsString]) -> Result<i32> {
    let mut scan = false;
    for a in args {
        let b = ab(a);
        if b == b"-s" {
            scan = true;
        } else {
            eprintln!("mdev: invalid option '{}'", lossy(a));
            return Ok(1);
        }
    }
    if scan {
        let mut rc = 0;
        let mut stack = vec![
            String::from("/sys/devices"),
            String::from("/sys/class"),
            String::from("/sys/block"),
        ];
        let mut count = 0u32;
        while let Some(dir) = stack.pop() {
            let rd = match std::fs::read_dir(&dir) {
                Ok(r) => r,
                Err(_) => continue,
            };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() && !p.is_symlink() {
                    stack.push(p.to_string_lossy().into_owned());
                } else if p.file_name().is_some_and(|n| n == "uevent") {
                    if std::fs::write(&p, b"add").is_err() {
                        rc = 1;
                    } else {
                        count += 1;
                    }
                }
            }
            if count > 20000 {
                break;
            }
        }

        for sys in ["/sys/bus", "/sys/subsystem"] {
            if Path::new(sys).exists() {
                let _ = std::fs::write(format!("{}/uevent", sys), b"add");
            }
        }
        return Ok(rc);
    }

    let act = std::env::var("ACTION").unwrap_or_default();
    let devpath = std::env::var("DEVPATH").unwrap_or_default();
    if devpath.is_empty() {
        eprintln!("mdev: no DEVPATH in environment (run with -s or as hotplug helper)");
        return Ok(1);
    }
    if act == "remove" {
        let devname = std::env::var("DEVNAME").unwrap_or_default();
        if !devname.is_empty() {
            let _ = std::fs::remove_file(format!("/dev/{}", devname));
        }
        return Ok(0);
    }

    let uev = std::fs::read_to_string(format!("/sys{}/uevent", devpath)).unwrap_or_default();
    let mut major: Option<u32> = None;
    let mut minor: Option<u32> = None;
    let mut mode = 0o660u32;
    for line in uev.lines() {
        if let Some(v) = line.strip_prefix("MAJOR=") {
            major = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("MINOR=") {
            minor = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("DEVMODE=") {
            mode = u32::from_str_radix(v.trim().trim_start_matches("0o"), 8).unwrap_or(0o660);
        }
    }
    let devname = std::env::var("DEVNAME")
        .unwrap_or_else(|_| devpath.rsplit('/').next().unwrap_or("node").to_string());
    let (maj, min) = match (major, minor) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            eprintln!("mdev: no MAJOR/MINOR for {}", devpath);
            return Ok(1);
        }
    };
    let is_block = uev.contains("DEVTYPE=disk")
        || uev.contains("DEVTYPE=partition")
        || std::env::var("SUBSYSTEM").unwrap_or_default() == "block";
    let ftype = if is_block {
        libc::S_IFBLK
    } else {
        libc::S_IFCHR
    };
    let dst = format!("/dev/{}", devname);
    let c = match std::ffi::CString::new(dst.as_bytes()) {
        Ok(c) => c,
        Err(_) => return Ok(1),
    };
    let dev = libc::makedev(maj, min);
    if unsafe { libc::mknod(c.as_ptr(), ftype | mode, dev) } != 0 {
        let e = std::io::Error::last_os_error();
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            eprintln!("mdev: mknod '{}' failed: {}", dst, e);
            return Ok(1);
        }
    }
    let _ = std::fs::write(format!("/sys{}/uevent", devpath), b"add");
    Ok(0)
}
