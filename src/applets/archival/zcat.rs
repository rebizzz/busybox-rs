use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::Path;

pub struct ZcatApplet;
impl Applet for ZcatApplet {
    fn name(&self) -> &'static str {
        "zcat"
    }
    fn description(&self) -> &'static str {
        "Decompress gzip members to stdout; non-gzip input is copied through"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let files: Vec<&OsString> = args
            .iter()
            .filter(|a| ab(a) != b"-f" && ab(a) != b"--force")
            .collect();
        let mut rc = 0;
        let out = std::io::stdout();
        let mut lock = out.lock();
        if files.is_empty() {
            let mut data = Vec::new();
            if std::io::stdin().read_to_end(&mut data).is_err() {
                return Ok(1);
            }
            let payload = if is_gzip(&data) {
                match decode_gzip_members(&data) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("zcat: {e}");
                        return Ok(1);
                    }
                }
            } else {
                data
            };
            if lock.write_all(&payload).is_err() {
                return Ok(1);
            }
            return Ok(0);
        }
        for f in files {
            let data = match std::fs::read(Path::new(f)) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("zcat: {}: {e}", f.to_string_lossy());
                    rc = 1;
                    continue;
                }
            };

            let payload = if is_gzip(&data) {
                match decode_gzip_members(&data) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("zcat: {}: {e}", f.to_string_lossy());
                        rc = 1;
                        continue;
                    }
                }
            } else {
                data
            };
            if lock.write_all(&payload).is_err() {
                return Ok(1);
            }
        }
        Ok(rc)
    }
}
