use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::io::{self};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct MkfifoApplet;

impl Applet for MkfifoApplet {
    fn name(&self) -> &'static str {
        "mkfifo"
    }
    fn description(&self) -> &'static str {
        "Create named pipes (FIFOs)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mode = 0o666;
        let mut files = Vec::new();

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
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }

        if files.is_empty() {
            eprintln!("mkfifo: missing operand");
            return Ok(1);
        }

        let mut ret = 0;
        for path in files {
            let cpath = match CString::new(path.as_os_str().as_bytes()) {
                Ok(c) => c,
                Err(_) => {
                    ret = 1;
                    continue;
                }
            };

            let res = unsafe { libc::mkfifo(cpath.as_ptr(), mode as libc::mode_t) };
            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("mkfifo: cannot create fifo '{}': {}", path.display(), err);
                ret = 1;
            }
        }

        Ok(ret)
    }
}

