use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct RmdirApplet;
impl Applet for RmdirApplet {
    fn name(&self) -> &'static str {
        "rmdir"
    }
    fn description(&self) -> &'static str {
        "Remove EMPTY DIRECTORY(ies)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut parents = false;
        let mut dirs = Vec::new();
        for arg in args {
            let b = arg.as_bytes();
            if b.starts_with(b"-") && b != b"-" {
                if b == b"-p" || b == b"--parents" {
                    parents = true;
                } else if b.starts_with(b"-") {
                    for &c in &b[1..] {
                        if c == b'p' {
                            parents = true;
                        }
                    }
                }
            } else {
                dirs.push(Path::new(arg));
            }
        }

        let mut exit_code = 0;
        for d in dirs {
            let mut curr = d;
            loop {
                if let Err(e) = fs::remove_dir(curr) {
                    eprintln!("rmdir: failed to remove '{}': {}", curr.display(), e);
                    exit_code = 1;
                    break;
                }
                if parents {
                    if let Some(parent) = curr.parent() {
                        if !parent.as_os_str().is_empty() && parent != Path::new(".") {
                            curr = parent;
                            continue;
                        }
                    }
                }
                break;
            }
        }
        Ok(exit_code)
    }
}
