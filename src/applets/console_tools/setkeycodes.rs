use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self};
use std::os::unix::ffi::OsStrExt;

pub struct SetkeycodesApplet;

impl Applet for SetkeycodesApplet {
    fn name(&self) -> &'static str {
        "setkeycodes"
    }

    fn description(&self) -> &'static str {
        "Set kernel scancode-to-keycode mapping"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        #[allow(clippy::manual_is_multiple_of)]
        if args.len() < 3 || (args.len() - 1) % 2 != 0 {
            eprintln!("Usage: setkeycodes SCANCODE KEYCODE ...");
            return Ok(1);
        }

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("setkeycodes: {}", e);
                return Ok(1);
            }
        };

        let mut i = 1;
        while i < args.len() {
            let scan_str = args[i].as_bytes();
            let key_str = args[i + 1].as_bytes();
            let scan = if scan_str.starts_with(b"0x") || scan_str.starts_with(b"0X") {
                u32::from_str_radix(std::str::from_utf8(&scan_str[2..]).unwrap_or(""), 16).ok()
            } else {
                parse_u32(scan_str)
            };
            let key = parse_u32(key_str);

            match (scan, key) {
                (Some(s), Some(k)) => {
                    let mut a = Kbkeycode {
                        scancode: s,
                        keycode: k,
                    };
                    let ret = unsafe {
                        libc::ioctl(
                            console.as_raw_fd(),
                            KDSETKEYCODE,
                            &mut a as *mut Kbkeycode as *mut libc::c_void,
                        )
                    };
                    if ret < 0 {
                        eprintln!(
                            "setkeycodes: failed setting scancode {} to {}: {}",
                            s,
                            k,
                            io::Error::last_os_error()
                        );
                        return Ok(1);
                    }
                }
                _ => {
                    eprintln!("setkeycodes: invalid scancode or keycode");
                    return Ok(1);
                }
            }
            i += 2;
        }
        Ok(0)
    }
}
