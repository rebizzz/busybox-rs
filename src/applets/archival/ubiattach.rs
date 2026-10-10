use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::os::unix::io::AsRawFd;

fn ubi_ctrl_open(given: Option<&str>) -> std::io::Result<(File, String)> {
    let dev = given.unwrap_or("/dev/ubi_ctrl").to_owned();
    open_dev(&dev).map(|f| (f, dev))
}

pub struct UbiattachApplet;
impl Applet for UbiattachApplet {
    fn name(&self) -> &'static str {
        "ubiattach"
    }
    fn description(&self) -> &'static str {
        "Attach MTD to UBI (UBI_CTRL_IOCATT; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mtd: Option<i32> = None;
        let mut ubi: i32 = -1;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if (b == b"-m" || b == b"--mtdn") && i + 1 < args.len() {
                mtd = String::from_utf8_lossy(ab(&args[i + 1])).parse().ok();
                i += 1;
            } else if (b == b"-d" || b == b"--devn") && i + 1 < args.len() {
                ubi = String::from_utf8_lossy(ab(&args[i + 1]))
                    .parse()
                    .unwrap_or(-1);
                i += 1;
            }
            i += 1;
        }
        let mtd = match mtd {
            Some(m) => m,
            None => {
                eprintln!("ubiattach: missing -m MTD number");
                return Ok(1);
            }
        };
        let (f, dev) = match ubi_ctrl_open(None) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("ubiattach: /dev/ubi_ctrl: {e}");
                return Ok(1);
            }
        };
        let req_data = UbiAttachReq {
            ubi_num: ubi,
            mtd_num: mtd,
            vid_hdr_offset: 0,
            padding: [0; 12],
        };

        let code = iow(b'O', 64, size_of::<UbiAttachReq>());
        let r = unsafe { libc::ioctl(f.as_raw_fd(), code, &req_data) };
        if r != 0 {
            return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
        }
        Ok(0)
    }
}
