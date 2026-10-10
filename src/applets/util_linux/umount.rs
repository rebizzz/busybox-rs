use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::io::{self, BufRead};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

fn umount_one(target: &[u8], flags: libc::c_int, retry_ro: bool) -> i32 {
    let p = Path::new(std::ffi::OsStr::from_bytes(target));
    let c = match path_cstr(p) {
        Some(c) => c,
        None => {
            eprintln!("umount: bad path");
            return 1;
        }
    };

    if unsafe { libc::umount2(c.as_ptr(), flags) } == 0 {
        return 0;
    }
    let e = io::Error::last_os_error();
    if retry_ro {
        let r = unsafe {
            libc::mount(
                c.as_ptr(),
                c.as_ptr(),
                std::ptr::null(),
                (libc::MS_REMOUNT | libc::MS_RDONLY) as libc::c_ulong,
                std::ptr::null(),
            )
        };
        if r == 0 {
            return 0;
        }
    }
    eprintln!("umount: {}: {}", String::from_utf8_lossy(target), e);
    1
}

pub struct UmountApplet;
impl Applet for UmountApplet {
    fn name(&self) -> &'static str {
        "umount"
    }
    fn description(&self) -> &'static str {
        "Unmount filesystems"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut all = false;
        let mut retry_ro = false;
        let mut lazy = false;
        let mut force = false;
        let mut types: Vec<u8> = Vec::new();
        let mut pos: Vec<&[u8]> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for a in &args[i + 1..] {
                    pos.push(a.as_bytes());
                }
                break;
            } else if b.len() > 1 && b.starts_with(b"-") && !b.starts_with(b"--") {
                let mut j = 1;
                while j < b.len() {
                    match b[j] {
                        b'a' => all = true,
                        b'r' => retry_ro = true,
                        b'l' => lazy = true,
                        b'f' => force = true,
                        b'n' => {}
                        b'd' => {}
                        b't' => {
                            let rest = &b[j + 1..];
                            if rest.is_empty() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("umount: option requires an argument -- 't'");
                                    return Ok(1);
                                }
                                types = args[i].as_bytes().to_vec();
                            } else {
                                types = rest.to_vec();
                            }
                            break;
                        }
                        c => {
                            eprintln!("umount: invalid option -- '{}'", c as char);
                            return Ok(1);
                        }
                    }
                    j += 1;
                }
            } else {
                pos.push(b);
            }
            i += 1;
        }
        let mut flags: libc::c_int = 0;
        if lazy {
            flags |= libc::MNT_DETACH;
        }
        if force {
            flags |= libc::MNT_FORCE;
        }
        let type_ok = |fs: &[u8]| types.is_empty() || types.split(|&b| b == b',').any(|t| t == fs);
        if all {
            let mut rc = 0;
            let mounts = read_mounts();
            for (dev, mnt, fs) in mounts.iter().rev() {
                if mnt == b"/" || mnt == b"/proc" || mnt == b"/sys" || mnt == b"/dev" {
                    continue;
                }
                if !type_ok(fs) {
                    continue;
                }

                if umount_one(mnt, flags, retry_ro) != 0 {
                    rc |= umount_one(dev, flags, false);
                } else {
                    let _ = rc;
                }
            }
            return Ok(rc);
        }
        if pos.is_empty() {
            eprintln!("umount: no target");
            return Ok(1);
        }
        let mut rc = 0;
        for t in pos {
            rc |= umount_one(t, flags, retry_ro);
        }
        Ok(rc)
    }
}
