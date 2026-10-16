use super::readlink::canonicalize_coreutils;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct RealpathApplet;

impl Applet for RealpathApplet {
    fn name(&self) -> &'static str {
        "realpath"
    }

    fn description(&self) -> &'static str {
        "Print absolute pathnames of FILEs"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("Usage: realpath FILE...");
            return Ok(1);
        }

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut exit_code = 0;

        for arg in args {
            match canonicalize_coreutils(Path::new(arg)) {
                Some(p) => {
                    out.write_all(p.as_os_str().as_bytes())?;
                    out.write_all(b"\n")?;
                }
                None => {
                    exit_code = 1;
                    eprintln!(
                        "realpath: {}: No such file or directory",
                        arg.to_string_lossy()
                    );
                }
            }
        }

        let _ = out.flush();
        Ok(exit_code)
    }
}
