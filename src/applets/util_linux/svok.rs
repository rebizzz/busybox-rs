use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::io::{self};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct SvokApplet;
impl Applet for SvokApplet {
    fn name(&self) -> &'static str {
        "svok"
    }
    fn description(&self) -> &'static str {
        "Check whether runsv supervisor is running"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("svok: SERVICE_DIR required");
            return Ok(111);
        }

        let target = &args[0];
        let service_dir = Path::new(target);
        if !service_dir.exists() {
            eprintln!("svok: {}: file does not exist", service_dir.display());
            return Ok(111);
        }

        let ok_pipe = service_dir.join("supervise/ok");
        let c_path = match CString::new(ok_pipe.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(111),
        };

        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_WRONLY | libc::O_NONBLOCK) };
        if fd >= 0 {
            unsafe { libc::close(fd) };
            Ok(0)
        } else {
            let err = io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::ENXIO) || err.raw_os_error() == Some(libc::ENOENT) {
                Ok(100)
            } else {
                Ok(111)
            }
        }
    }
}
