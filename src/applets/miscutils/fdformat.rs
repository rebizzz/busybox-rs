use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct FdformatApplet;

impl Applet for FdformatApplet {
    fn name(&self) -> &'static str {
        "fdformat"
    }

    fn description(&self) -> &'static str {
        "Format a floppy disk"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut no_verify = false;
        let mut dev = None;
        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-n" {
                no_verify = true;
            } else if !b.starts_with(b"-") {
                dev = Some(&args[i]);
            }
            i += 1;
        }

        let dev_path = match dev {
            Some(d) => Path::new(d),
            None => {
                eprintln!("Usage: fdformat [-n] DEVICE");
                return Ok(1);
            }
        };

        let f = match OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NDELAY)
            .open(dev_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fdformat: {}: {}", dev_path.display(), e);
                return Ok(1);
            }
        };

        print!("Formatting... ");
        let _ = io::stdout().flush();
        for track in 0..80 {
            for head in 0..2 {
                let mut descr = FormatDescr {
                    device: 0,
                    head,
                    track,
                };
                let ret = unsafe {
                    libc::ioctl(
                        f.as_raw_fd(),
                        FDFMTTRK,
                        &mut descr as *mut FormatDescr as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    println!("failed");
                    eprintln!(
                        "fdformat: format track {} head {} failed: {}",
                        track,
                        head,
                        io::Error::last_os_error()
                    );
                    return Ok(1);
                }
            }
        }
        println!("done");

        if !no_verify {
            print!("Verifying... ");
            let _ = io::stdout().flush();
            let mut f_read = match File::open(dev_path) {
                Ok(fr) => fr,
                Err(e) => {
                    eprintln!("fdformat: verify open failed: {}", e);
                    return Ok(1);
                }
            };
            let mut buf = [0u8; 1024];
            while let Ok(n) = f_read.read(&mut buf) {
                if n == 0 {
                    break;
                }
            }
            println!("done");
        }

        Ok(0)
    }
}
