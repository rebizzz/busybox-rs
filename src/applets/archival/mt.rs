use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::io::AsRawFd;

pub struct MtApplet;
impl Applet for MtApplet {
    fn name(&self) -> &'static str {
        "mt"
    }
    fn description(&self) -> &'static str {
        "Tape drive control (MTIOCTOP ioctl; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = std::env::var_os("TAPE")
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from("/dev/tape"));
        let mut op: Option<i16> = None;
        let mut count: i32 = 1;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-f" && i + 1 < args.len() {
                dev = String::from_utf8_lossy(ab(&args[i + 1])).into_owned();
                i += 2;
                continue;
            }
            let s = String::from_utf8_lossy(b).to_lowercase();
            let code: Option<i16> = match s.as_str() {
                "eof" | "weof" => Some(5),
                "fsf" => Some(1),
                "bsf" => Some(2),
                "fsr" => Some(3),
                "bsr" => Some(4),
                "rewind" | "rew" => Some(6),
                "offline" | "eject" | "rewoffl" => Some(7),
                "retension" => Some(9),
                "erase" => Some(10),
                "eom" | "seod" => Some(11),
                "status" => Some(-1),
                _ => None,
            };
            if let Some(c) = code {
                op = Some(c);
            } else if let Ok(n) = s.parse::<i32>() {
                count = n;
            }
            i += 1;
        }
        let op = match op {
            Some(o) => o,
            None => {
                eprintln!("mt: missing operation (eof fsf bsf rewind offline status ...)");
                return Ok(1);
            }
        };
        let f = match open_dev(&dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("mt: {dev}: {e}");
                return Ok(1);
            }
        };
        if op == -1 {
            let mut buf = [0u8; 48];
            let code = ior(b'm', 2, buf.len());
            if unsafe { libc::ioctl(f.as_raw_fd(), code, buf.as_mut_ptr()) } != 0 {
                return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
            }
            println!("mt: status ok ({dev})");
            return Ok(0);
        }
        let m = Mtop { op, count };
        let code = iow(b'm', 1, size_of::<Mtop>());
        if unsafe { libc::ioctl(f.as_raw_fd(), code, &m) } != 0 {
            return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
        }
        Ok(0)
    }
}
