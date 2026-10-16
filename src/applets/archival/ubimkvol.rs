use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;

fn ubi_vol_ioctl(dev: &str, nr: u8, payload: &[u8]) -> i32 {
    let f = match open_dev(dev) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{dev}: {e}");
            return 1;
        }
    };

    let code = iow(b'o', nr, payload.len());
    let r = unsafe { libc::ioctl(f.as_raw_fd(), code, payload.as_ptr()) };
    if r != 0 {
        return ioctl_err(dev, std::io::Error::last_os_error());
    }
    0
}

pub struct UbimkvolApplet;
impl Applet for UbimkvolApplet {
    fn name(&self) -> &'static str {
        "ubimkvol"
    }
    fn description(&self) -> &'static str {
        "Create UBI volume (UBI_IOCMKVOL; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = String::from("/dev/ubi0");
        let mut name = String::new();
        let mut size: i64 = 0;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-N" && i + 1 < args.len() {
                name = String::from_utf8_lossy(ab(&args[i + 1])).into_owned();
                i += 1;
            } else if b == b"-s" && i + 1 < args.len() {
                size = String::from_utf8_lossy(ab(&args[i + 1]))
                    .parse()
                    .unwrap_or(0);
                i += 1;
            } else if !b.starts_with(b"-") {
                dev = String::from_utf8_lossy(b).into_owned();
            }
            i += 1;
        }
        if name.is_empty() {
            eprintln!("ubimkvol: missing -N name");
            return Ok(1);
        }

        let mut req = vec![0u8; 152];
        req[0..4].copy_from_slice(&(-1i32).to_le_bytes());
        req[4..8].copy_from_slice(&1i32.to_le_bytes());
        req[8..16].copy_from_slice(&size.to_le_bytes());
        req[16] = 3;
        let nb = name.as_bytes();
        let nl = nb.len().min(127) as i16;
        req[20..22].copy_from_slice(&nl.to_le_bytes());
        req[24..24 + nl as usize].copy_from_slice(&nb[..nl as usize]);
        Ok(ubi_vol_ioctl(&dev, 0, &req))
    }
}
