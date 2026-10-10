use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;

pub struct AddShellApplet;
impl Applet for AddShellApplet {
    fn name(&self) -> &'static str {
        "add-shell"
    }
    fn description(&self) -> &'static str {
        "Add a shell to /etc/shells"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        for a in args {
            if is_help(a) {
                return help_out("add-shell", "SHELL...", "Add SHELLs to /etc/shells");
            }
        }
        if args.is_empty() {
            eprintln!("Usage: add-shell SHELL...");
            return Ok(1);
        }
        let mut shells = read_shells();
        for a in args {
            let sh = a.as_bytes();
            if !sh.starts_with(b"/") {
                eprintln!("add-shell: shell must be an absolute path");
                return Ok(1);
            }
            let sp = std::path::Path::new(std::ffi::OsStr::from_bytes(sh));
            match fs::metadata(sp) {
                Ok(m) if m.is_file() => {}
                _ => {
                    eprintln!("add-shell: '{}' is not a file", String::from_utf8_lossy(sh));
                    return Ok(1);
                }
            }
            if !shells.iter().any(|s| s == sh) {
                shells.push(sh.to_vec());
            }
        }
        let mut out = Vec::new();
        for s in &shells {
            out.extend_from_slice(s);
            out.push(b'\n');
        }
        if let Err(e) = atomic_replace(&shells_path(), &out) {
            eprintln!(
                "add-shell: cannot update {}: {}",
                shells_path().display(),
                e
            );
            return Ok(1);
        }
        Ok(0)
    }
}
