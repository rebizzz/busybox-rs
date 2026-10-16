use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct UbirmvolApplet;
impl Applet for UbirmvolApplet {
    fn name(&self) -> &'static str {
        "ubirmvol"
    }
    fn description(&self) -> &'static str {
        "Remove UBI volume (UBI_IOCRMVOL; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = String::from("/dev/ubi0");
        let mut name = String::new();
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-N" && i + 1 < args.len() {
                name = String::from_utf8_lossy(ab(&args[i + 1])).into_owned();
                i += 1;
            } else if !b.starts_with(b"-") {
                dev = String::from_utf8_lossy(b).into_owned();
            }
            i += 1;
        }
        if name.is_empty() {
            eprintln!("ubirmvol: missing -N name");
            return Ok(1);
        }
        let mut req = vec![0u8; 136];
        let nb = name.as_bytes();
        req[0..2].copy_from_slice(&(nb.len().min(127) as i16).to_le_bytes());
        req[8..8 + nb.len().min(127)].copy_from_slice(&nb[..nb.len().min(127)]);
        Ok(ubi_vol_ioctl(&dev, 2, &req))
    }
}
