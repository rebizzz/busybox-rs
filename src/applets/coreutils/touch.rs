use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct TouchApplet;
impl Applet for TouchApplet {
    fn name(&self) -> &'static str {
        "touch"
    }
    fn description(&self) -> &'static str {
        "Update the access and modification times of each FILE to the current time"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut no_create = false;
        let mut files = Vec::new();
        for arg in args {
            let b = arg.as_bytes();
            if b == b"-c" || b == b"--no-create" {
                no_create = true;
            } else if !b.starts_with(b"-") || b == b"-" {
                files.push(Path::new(arg));
            }
        }

        let mut exit_code = 0;
        for f in files {
            if !f.exists() {
                if no_create {
                    continue;
                }
                if let Err(e) = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(false)
                    .open(f)
                {
                    eprintln!("touch: cannot touch '{}': {}", f.display(), e);
                    exit_code = 1;
                    continue;
                }
            }
            if let Ok(c_path) = std::ffi::CString::new(f.as_os_str().as_bytes()) {
                unsafe {
                    libc::utimensat(libc::AT_FDCWD, c_path.as_ptr(), std::ptr::null(), 0);
                }
            }
        }
        Ok(exit_code)
    }
}
