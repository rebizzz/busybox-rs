use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;

pub struct MktempApplet;

impl Applet for MktempApplet {
    fn name(&self) -> &'static str {
        "mktemp"
    }
    fn description(&self) -> &'static str {
        "Create temporary file or directory"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut directory = false;
        let mut quiet = false;
        let mut template: Option<Vec<u8>> = None;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-d" || b == b"--directory" {
                directory = true;
            } else if b == b"-q" || b == b"--quiet" {
                quiet = true;
            } else if !b.starts_with(b"-") {
                template = Some(b.to_vec());
            }
        }

        let templ = template.unwrap_or_else(|| b"/tmp/tmp.XXXXXX".to_vec());
        let mut ctempl = templ.clone();
        if !ctempl.ends_with(b"XXXXXX") {
            ctempl.extend_from_slice(b".XXXXXX");
        }
        ctempl.push(0);

        let ptr = ctempl.as_mut_ptr() as *mut libc::c_char;

        let res = unsafe {
            if directory {
                if libc::mkdtemp(ptr).is_null() {
                    -1
                } else {
                    0
                }
            } else {
                let fd = libc::mkstemp(ptr);
                if fd >= 0 {
                    libc::close(fd);
                    0
                } else {
                    -1
                }
            }
        };

        if res < 0 {
            if !quiet {
                eprintln!("mktemp: failed to create temporary file/directory");
            }
            return Ok(1);
        }

        ctempl.pop();
        let out = io::stdout();
        let mut lock = out.lock();
        lock.write_all(&ctempl)?;
        lock.write_all(b"\n")?;

        Ok(0)
    }
}

