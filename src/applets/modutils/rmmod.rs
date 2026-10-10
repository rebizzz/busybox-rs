use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::io::{self};
use std::os::unix::ffi::OsStrExt;

pub struct RmmodApplet;

impl Applet for RmmodApplet {
    fn name(&self) -> &'static str {
        "rmmod"
    }
    fn description(&self) -> &'static str {
        "Unload kernel module"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut force = false;
        let mut mod_name: Option<&[u8]> = None;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-f" || b == b"--force" {
                force = true;
            } else if !b.is_empty() && b[0] == b'-' {
            } else if mod_name.is_none() {
                mod_name = Some(b);
            }
        }

        let name = match mod_name {
            Some(n) => n,
            None => {
                eprintln!("rmmod: module name required");
                return Ok(1);
            }
        };

        let name_cstr = match CString::new(name) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("rmmod: invalid module name");
                return Ok(1);
            }
        };

        let flags = if force {
            libc::O_TRUNC | libc::O_NONBLOCK
        } else {
            libc::O_NONBLOCK
        };
        let ret = unsafe { libc::syscall(libc::SYS_delete_module, name_cstr.as_ptr(), flags) };

        if ret != 0 {
            let err = io::Error::last_os_error();
            eprintln!(
                "rmmod: can't unload '{}': {}",
                String::from_utf8_lossy(name),
                err
            );
            return Ok(1);
        }

        Ok(0)
    }
}
