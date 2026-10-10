use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct RmApplet;
impl Applet for RmApplet {
    fn name(&self) -> &'static str {
        "rm"
    }
    fn description(&self) -> &'static str {
        "Remove (unlink) the FILE(s)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut recursive = false;
        let mut force = false;
        let mut paths = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                for &b in &bytes[1..] {
                    match b {
                        b'r' | b'R' => recursive = true,
                        b'f' => force = true,
                        _ => {}
                    }
                }
            } else {
                paths.push(Path::new(arg));
            }
        }

        let mut exit_code = 0;
        for p in paths {
            let res = if recursive {
                if p.is_dir() {
                    fs::remove_dir_all(p)
                } else {
                    fs::remove_file(p)
                }
            } else {
                fs::remove_file(p)
            };

            if let Err(e) = res {
                if !force {
                    eprintln!("rm: cannot remove '{}': {}", p.display(), e);
                    exit_code = 1;
                }
            }
        }
        Ok(exit_code)
    }
}
