use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::{CString, OsString};
use std::io::{self};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;

pub struct SetserialApplet;
impl Applet for SetserialApplet {
    fn name(&self) -> &'static str {
        "setserial"
    }
    fn description(&self) -> &'static str {
        "Retrieve or set Linux serial port"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("setserial: DEVICE [PARAMETERS]");
            return Ok(1);
        }

        let dev = &args[0];
        let c_path = match CString::new(dev.as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDWR | libc::O_NONBLOCK) };
        if fd < 0 {
            let err = io::Error::last_os_error();
            eprintln!("setserial: {}: {}", dev.to_string_lossy(), err);
            return Ok(1);
        }

        let mut serinfo = MaybeUninit::<SerialStruct>::zeroed();
        let res = unsafe { libc::ioctl(fd, TIOCGSERIAL, serinfo.as_mut_ptr()) };
        if res != 0 {
            unsafe { libc::close(fd) };
            let err = io::Error::last_os_error();
            eprintln!("setserial: {}: {}", dev.to_string_lossy(), err);
            return Ok(1);
        }
        let mut serinfo = unsafe { serinfo.assume_init() };

        if args.len() == 1 {
            println!(
                "{}, Line {}, UART: {}, Port: 0x{:x}, IRQ: {}",
                dev.to_string_lossy(),
                serinfo.line,
                serinfo.type_,
                serinfo.port,
                serinfo.irq
            );
            unsafe { libc::close(fd) };
            return Ok(0);
        }

        let mut i = 1;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "port" && i + 1 < args.len() {
                let p_str = args[i + 1].to_string_lossy();
                if let Some(hex) = p_str.strip_prefix("0x") {
                    serinfo.port = u32::from_str_radix(hex, 16).unwrap_or(serinfo.port);
                } else {
                    serinfo.port = p_str.parse().unwrap_or(serinfo.port);
                }
                i += 2;
                continue;
            } else if s == "irq" && i + 1 < args.len() {
                serinfo.irq = args[i + 1].to_string_lossy().parse().unwrap_or(serinfo.irq);
                i += 2;
                continue;
            } else if s == "baud_base" && i + 1 < args.len() {
                serinfo.baud_base = args[i + 1]
                    .to_string_lossy()
                    .parse()
                    .unwrap_or(serinfo.baud_base);
                i += 2;
                continue;
            }
            i += 1;
        }

        let set_res = unsafe { libc::ioctl(fd, TIOCSSERIAL, &serinfo) };
        unsafe { libc::close(fd) };

        if set_res != 0 {
            let err = io::Error::last_os_error();
            eprintln!("setserial: cannot set parameters: {}", err);
            return Ok(1);
        }

        Ok(0)
    }
}
