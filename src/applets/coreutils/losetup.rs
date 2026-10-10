use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

applet!(LosetupApplet, "losetup", "Set up loop devices", run_losetup);
fn run_losetup(args: &[OsString]) -> Result<i32> {
    let (mut show_all, mut find_free, mut detach) = (false, false, false);
    let mut rest: Vec<OsString> = Vec::new();
    for a in args {
        let b = ab(a);
        if b == b"-a" || b == b"--list" {
            show_all = true;
        } else if b == b"-f" || b == b"--find" {
            find_free = true;
        } else if b == b"-d" || b == b"--detach" {
            detach = true;
        } else if b == b"-o" || b == b"--offset" {
            eprintln!("losetup: -o/--offset is not supported in this build");
            return Ok(1);
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("losetup: invalid option '{}'", lossy(a));
            return Ok(1);
        } else {
            rest.push(a.clone());
        }
    }
    use std::os::unix::io::AsRawFd;
    if show_all || (rest.is_empty() && !find_free && !detach) {
        let mut out = wlock();
        let mut n = 0u32;
        loop {
            let p = format!("/dev/loop{}", n);
            if Path::new(&p).exists() || Path::new(&format!("/sys/block/loop{}", n)).exists() {
                let bf = std::fs::read_to_string(format!("/sys/block/loop{}/loop/backing_file", n))
                    .unwrap_or_default();
                let bf = bf.trim();
                if !bf.is_empty() {
                    let _ = writeln!(out, "{}: {}", p, bf);
                }
                n += 1;
                if n > 256 {
                    break;
                }
            } else if n > 8 {
                break;
            } else {
                n += 1;
            }
        }
        return Ok(0);
    }
    if find_free && rest.is_empty() {
        let mut n = 0u32;
        loop {
            let p = format!("/dev/loop{}", n);
            let bf = std::fs::read_to_string(format!("/sys/block/loop{}/loop/backing_file", n))
                .unwrap_or_default();
            if bf.trim().is_empty()
                && (Path::new(&p).exists() || Path::new(&format!("/sys/block/loop{}", n)).exists())
            {
                println!("{}", p);
                return Ok(0);
            }
            n += 1;
            if n > 256 {
                eprintln!("losetup: no free loop device");
                return Ok(1);
            }
        }
    }
    if detach {
        if rest.is_empty() {
            eprintln!("losetup: -d needs a device");
            return Ok(1);
        }
        let mut rc = 0;
        for d in &rest {
            match File::options().read(true).write(true).open(Path::new(d)) {
                Ok(f) => {
                    if unsafe { libc::ioctl(f.as_raw_fd(), 0x4c01u64 as _) } < 0 {
                        eprintln!(
                            "losetup: detach '{}' failed: {}",
                            lossy(d),
                            std::io::Error::last_os_error()
                        );
                        rc = 1;
                    }
                }
                Err(e) => {
                    eprintln!("losetup: can't open '{}': {}", lossy(d), e);
                    rc = 1;
                }
            }
        }
        return Ok(rc);
    }

    if rest.len() != 2 {
        eprintln!("losetup: usage: losetup [-a|-f|-d DEV] [DEV FILE]");
        return Ok(1);
    }
    let (dev, file) = (&rest[0], &rest[1]);
    let lf = match File::options().read(true).write(true).open(Path::new(dev)) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("losetup: can't open '{}': {}", lossy(dev), e);
            return Ok(1);
        }
    };
    let bf = match File::open(Path::new(file)) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("losetup: can't open '{}': {}", lossy(file), e);
            return Ok(1);
        }
    };
    if unsafe { libc::ioctl(lf.as_raw_fd(), 0x4c00u64 as _, bf.as_raw_fd()) } < 0 {
        eprintln!(
            "losetup: attach failed: {}",
            std::io::Error::last_os_error()
        );
        return Ok(1);
    }
    Ok(0)
}
