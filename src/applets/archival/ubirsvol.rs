use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct UbirsvolApplet;
impl Applet for UbirsvolApplet {
    fn name(&self) -> &'static str {
        "ubirsvol"
    }
    fn description(&self) -> &'static str {
        "Resize UBI volume (UBI_IOCRSVOL; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = String::from("/dev/ubi0_0");
        let mut size: i64 = 0;
        for a in args {
            let b = ab(a);
            if b.starts_with(b"-") {
                continue;
            }
            let s = String::from_utf8_lossy(b).into_owned();
            if let Ok(n) = s.parse::<i64>() {
                size = n;
            } else {
                dev = s;
            }
        }

        let mut req = vec![0u8; 16];
        req[8..16].copy_from_slice(&size.to_le_bytes());
        Ok(ubi_vol_ioctl(&dev, 3, &req))
    }
}
