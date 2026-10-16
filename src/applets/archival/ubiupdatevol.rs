use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

pub struct UbiupdatevolApplet;
impl Applet for UbiupdatevolApplet {
    fn name(&self) -> &'static str {
        "ubiupdatevol"
    }
    fn description(&self) -> &'static str {
        "Update UBI volume from stdin/file (UBI_IOCVOLUP; graceful error w/o hw)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev: Option<String> = None;
        let mut input: Option<PathBuf> = None;
        for a in args {
            let b = ab(a);
            if b == b"-t" || b.starts_with(b"-") {
                continue;
            }
            if dev.is_none() {
                dev = Some(String::from_utf8_lossy(b).into_owned());
            } else {
                input = Some(PathBuf::from(a));
            }
        }
        let dev = dev.unwrap_or_else(|| String::from("/dev/ubi0_0"));
        let data = match read_all_bytes(input.as_deref()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("ubiupdatevol: input: {e}");
                return Ok(1);
            }
        };
        let f = match open_dev(&dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("ubiupdatevol: {dev}: {e}");
                return Ok(1);
            }
        };

        let mut req = [0u8; 16];
        req[8..16].copy_from_slice(&(data.len() as i64).to_le_bytes());
        let code = iow(b'o', 0, 16);
        if unsafe { libc::ioctl(f.as_raw_fd(), code, req.as_ptr()) } != 0 {
            return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
        }
        use std::os::unix::fs::FileExt;
        let mut off = 0u64;
        let mut left = data.as_slice();
        while !left.is_empty() {
            match f.write_at(left, off) {
                Ok(0) => break,
                Ok(n) => {
                    off += n as u64;
                    left = &left[n..];
                }
                Err(e) => {
                    eprintln!("ubiupdatevol: {dev}: {e}");
                    return Ok(1);
                }
            }
        }
        Ok(0)
    }
}
