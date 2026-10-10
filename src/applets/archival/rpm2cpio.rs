use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::path::Path;

use crate::applets::archival::rpm::*;
pub struct Rpm2cpioApplet;
impl Applet for Rpm2cpioApplet {
    fn name(&self) -> &'static str {
        "rpm2cpio"
    }
    fn description(&self) -> &'static str {
        "Write rpm payload as (decompressed) cpio archive to stdout"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("rpm2cpio: missing rpm file");
            return Ok(1);
        }
        let data = match std::fs::read(Path::new(&args[0])) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("rpm2cpio: {}: {e}", args[0].to_string_lossy());
                return Ok(1);
            }
        };
        let pkg = match parse_rpm(&data) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("rpm2cpio: {e}");
                return Ok(1);
            }
        };
        match rpm_payload_cpio(&pkg) {
            Ok(cpio) => {
                if stdout_write(&cpio).is_err() {
                    return Ok(1);
                }
                Ok(0)
            }
            Err(e) => {
                eprintln!("rpm2cpio: {e}");
                Ok(1)
            }
        }
    }
}
