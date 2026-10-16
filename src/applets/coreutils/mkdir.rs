use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct MkdirApplet;
impl Applet for MkdirApplet {
    fn name(&self) -> &'static str {
        "mkdir"
    }
    fn description(&self) -> &'static str {
        "Create the DIRECTORY(ies)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut parents = false;
        let mut dirs = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 {
                for &b in &bytes[1..] {
                    if b == b'p' {
                        parents = true;
                    }
                }
            } else {
                dirs.push(Path::new(arg));
            }
        }

        if dirs.is_empty() {
            eprintln!("mkdir: missing operand");
            return Ok(1);
        }

        let mut exit_code = 0;
        for d in dirs {
            let res = if parents {
                fs::create_dir_all(d)
            } else {
                fs::create_dir(d)
            };
            if let Err(e) = res {
                eprintln!("mkdir: cannot create directory '{}': {}", d.display(), e);
                exit_code = 1;
            }
        }
        Ok(exit_code)
    }
}
