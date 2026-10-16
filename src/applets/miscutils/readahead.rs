use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct ReadaheadApplet;

impl Applet for ReadaheadApplet {
    fn name(&self) -> &'static str {
        "readahead"
    }

    fn description(&self) -> &'static str {
        "Preload files into page cache"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: readahead FILE...");
            return Ok(1);
        }

        let mut ret_code = 0;
        for arg in &args[1..] {
            let path = Path::new(arg);
            let f = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("readahead: {}: {}", path.display(), e);
                    ret_code = 1;
                    continue;
                }
            };

            let len = f.metadata().map(|m| m.len()).unwrap_or(0);
            let fd = f.as_raw_fd();
            let ret = unsafe {
                libc::posix_fadvise(fd, 0, len as libc::off_t, libc::POSIX_FADV_WILLNEED)
            };
            if ret != 0 {
                unsafe {
                    libc::readahead(fd, 0, len as libc::size_t);
                }
            }
        }
        Ok(ret_code)
    }
}
