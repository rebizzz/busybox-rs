use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct FstrimApplet;

#[repr(C)]
struct FstrimRange {
    start: u64,
    len: u64,
    minlen: u64,
}

const FITRIM_IOCTL: libc::c_ulong = 0xc0185879;

impl Applet for FstrimApplet {
    fn name(&self) -> &'static str {
        "fstrim"
    }
    fn description(&self) -> &'static str {
        "Discard unused blocks on a mounted filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mount_point: Option<&Path> = None;
        let mut verbose = false;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-v" || b == b"--verbose" {
                verbose = true;
            } else if !b.is_empty() && b[0] == b'-' {
            } else if mount_point.is_none() {
                mount_point = Some(Path::new(arg));
            }
        }

        let mnt = match mount_point {
            Some(p) => p,
            None => {
                eprintln!("fstrim: mountpoint required");
                return Ok(1);
            }
        };

        let f = match File::open(mnt) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fstrim: {}: {}", mnt.display(), e);
                return Ok(1);
            }
        };

        let mut range = FstrimRange {
            start: 0,
            len: u64::MAX,
            minlen: 0,
        };

        let ret =
            unsafe { libc::ioctl(f.as_raw_fd(), FITRIM_IOCTL, &mut range as *mut FstrimRange) };

        if ret < 0 {
            let err = io::Error::last_os_error();
            eprintln!("fstrim: {}: FITRIM ioctl failed: {}", mnt.display(), err);
            return Ok(1);
        }

        if verbose {
            let mut out = Vec::new();
            out.extend_from_slice(mnt.as_os_str().as_bytes());
            out.extend_from_slice(b": ");
            put_num(&mut out, range.len);
            out.extend_from_slice(b" bytes trimmed\n");
            print_bytes(&out);
        }

        Ok(0)
    }
}
