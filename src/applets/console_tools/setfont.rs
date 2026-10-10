use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct SetfontApplet;

impl Applet for SetfontApplet {
    fn name(&self) -> &'static str {
        "setfont"
    }

    fn description(&self) -> &'static str {
        "Load EGA/VGA console screen font"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: setfont FONT_FILE");
            return Ok(1);
        }
        let font_path = &args[1];
        let mut font_data = Vec::new();
        let path = Path::new(font_path);
        match File::open(path) {
            Ok(mut f) => {
                let _ = f.read_to_end(&mut font_data);
            }
            Err(e) => {
                eprintln!("setfont: {}: {}", path.display(), e);
                return Ok(1);
            }
        }

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("setfont: {}", e);
                return Ok(1);
            }
        };

        let raw_font = if font_data.len() > 4 && font_data[0] == 0x36 && font_data[1] == 0x04 {
            let hdr_size = 4;
            &font_data[hdr_size..]
        } else {
            &font_data[..]
        };

        let ret = unsafe {
            libc::ioctl(
                console.as_raw_fd(),
                PIO_FONT,
                raw_font.as_ptr() as *const libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!("setfont: PIO_FONT failed: {}", io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}
