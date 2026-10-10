use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::io::{self};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct MknodApplet;

impl Applet for MknodApplet {
    fn name(&self) -> &'static str {
        "mknod"
    }
    fn description(&self) -> &'static str {
        "Create block or character special files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mode: u32 = 0o666;
        let mut pos = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-m" || b == b"--mode" {
                if i + 1 < args.len() {
                    i += 1;
                    mode = u32::from_str_radix(
                        std::str::from_utf8(args[i].as_bytes()).unwrap_or("666"),
                        8,
                    )
                    .unwrap_or(0o666);
                }
            } else if b.starts_with(b"-m") && b.len() > 2 {
                mode = u32::from_str_radix(std::str::from_utf8(&b[2..]).unwrap_or("666"), 8)
                    .unwrap_or(0o666);
            } else if b.starts_with(b"-") {
            } else {
                pos.push(&args[i]);
            }
            i += 1;
        }

        if pos.len() < 2 {
            eprintln!("mknod: missing operand");
            return Ok(1);
        }

        let path = Path::new(pos[0]);
        let dev_type = pos[1].as_bytes();

        let (node_type, dev) = match dev_type {
            b"p" => (libc::S_IFIFO, 0),
            b"c" | b"u" => {
                if pos.len() < 4 {
                    eprintln!("mknod: missing major and minor");
                    return Ok(1);
                }
                let major: u64 = std::str::from_utf8(pos[2].as_bytes())
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                let minor: u64 = std::str::from_utf8(pos[3].as_bytes())
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                let dev = libc::makedev(major as u32, minor as u32);
                (libc::S_IFCHR, dev)
            }
            b"b" => {
                if pos.len() < 4 {
                    eprintln!("mknod: missing major and minor");
                    return Ok(1);
                }
                let major: u64 = std::str::from_utf8(pos[2].as_bytes())
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                let minor: u64 = std::str::from_utf8(pos[3].as_bytes())
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                let dev = libc::makedev(major as u32, minor as u32);
                (libc::S_IFBLK, dev)
            }
            _ => {
                eprintln!("mknod: unknown device type");
                return Ok(1);
            }
        };

        let cpath = match CString::new(path.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let full_mode = node_type | mode;
        let res = unsafe {
            libc::mknod(
                cpath.as_ptr(),
                full_mode as libc::mode_t,
                dev as libc::dev_t,
            )
        };

        if res != 0 {
            let err = io::Error::last_os_error();
            eprintln!("mknod: cannot create node '{}': {}", path.display(), err);
            return Ok(1);
        }

        Ok(0)
    }
}
