use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

pub struct NandwriteApplet;
impl Applet for NandwriteApplet {
    fn name(&self) -> &'static str {
        "nandwrite"
    }
    fn description(&self) -> &'static str {
        "Write to NAND flash (MEMERASE+write; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev: Option<String> = None;
        let mut input: Option<PathBuf> = None;
        let mut start: u32 = 0;
        for a in args {
            let b = ab(a);

            if b.starts_with(b"-") {
                continue;
            }
            if dev.is_none() {
                dev = Some(String::from_utf8_lossy(b).into_owned());
            } else if input.is_none() {
                input = Some(PathBuf::from(a));
            }
        }

        let mut si = 0;
        while si < args.len() {
            if ab(&args[si]) == b"-s" && si + 1 < args.len() {
                start = String::from_utf8_lossy(ab(&args[si + 1]))
                    .parse()
                    .unwrap_or(0);
            }
            si += 1;
        }
        let dev = match dev {
            Some(d) => d,
            None => {
                eprintln!("nandwrite: missing mtd device");
                return Ok(1);
            }
        };
        let data = match read_all_bytes(input.as_deref()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("nandwrite: input: {e}");
                return Ok(1);
            }
        };
        let f = match open_dev(&dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("nandwrite: {dev}: {e}");
                return Ok(1);
            }
        };
        let erase = EraseInfoUser {
            start,
            length: data.len() as u32,
        };
        let req = iow(b'M', 2, size_of::<EraseInfoUser>());
        let r = unsafe { libc::ioctl(f.as_raw_fd(), req, &erase) };
        if r != 0 {
            eprintln!(
                "nandwrite: erase skipped: {}",
                std::io::Error::last_os_error()
            );
        }
        use std::os::unix::fs::FileExt;
        match f.write_at(&data, u64::from(start)) {
            Ok(n) => {
                eprintln!("nandwrite: wrote {n} bytes to {dev}");
                Ok(0)
            }
            Err(e) => {
                eprintln!("nandwrite: {dev}: {e}");
                Ok(1)
            }
        }
    }
}
