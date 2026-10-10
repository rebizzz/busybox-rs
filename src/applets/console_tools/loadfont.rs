use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct LoadfontApplet;

impl Applet for LoadfontApplet {
    fn name(&self) -> &'static str {
        "loadfont"
    }

    fn description(&self) -> &'static str {
        "Load console font"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut font_data = Vec::new();
        if args.len() <= 1 {
            let mut stdin = io::stdin().lock();
            let _ = stdin.read_to_end(&mut font_data);
        } else {
            let path = Path::new(&args[1]);
            match File::open(path) {
                Ok(mut f) => {
                    let _ = f.read_to_end(&mut font_data);
                }
                Err(e) => {
                    eprintln!("loadfont: {}: {}", path.display(), e);
                    return Ok(1);
                }
            }
        }

        if font_data.is_empty() {
            eprintln!("loadfont: empty font data");
            return Ok(1);
        }

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("loadfont: {}", e);
                return Ok(1);
            }
        };

        let ret = unsafe {
            libc::ioctl(
                console.as_raw_fd(),
                PIO_FONT,
                font_data.as_ptr() as *const libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!(
                "loadfont: PIO_FONT ioctl failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        Ok(0)
    }
}
