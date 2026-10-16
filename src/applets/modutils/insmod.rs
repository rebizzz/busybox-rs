use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, File};
use std::io::{self};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct InsmodApplet;

impl Applet for InsmodApplet {
    fn name(&self) -> &'static str {
        "insmod"
    }
    fn description(&self) -> &'static str {
        "Load kernel module into the kernel"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("insmod: filename required");
            return Ok(1);
        }

        let path = Path::new(&args[0]);
        let mut opts = Vec::new();
        for (i, a) in args[1..].iter().enumerate() {
            if i > 0 {
                opts.push(b' ');
            }
            opts.extend_from_slice(a.as_bytes());
        }
        let opts_cstr = CString::new(opts).unwrap_or_else(|_| CString::new("").unwrap());

        let f = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("insmod: can't open '{}': {}", path.display(), e);
                return Ok(1);
            }
        };

        let fd = f.as_raw_fd();
        let ret = unsafe { libc::syscall(libc::SYS_finit_module, fd, opts_cstr.as_ptr(), 0) };

        if ret == 0 {
            return Ok(0);
        }

        let data = match fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("insmod: can't read '{}': {}", path.display(), e);
                return Ok(1);
            }
        };

        let ret = unsafe {
            libc::syscall(
                libc::SYS_init_module,
                data.as_ptr() as *const libc::c_void,
                data.len() as libc::size_t,
                opts_cstr.as_ptr(),
            )
        };

        if ret != 0 {
            let err = io::Error::last_os_error();
            eprintln!("insmod: can't insert '{}': {}", path.display(), err);
            return Ok(1);
        }

        Ok(0)
    }
}
