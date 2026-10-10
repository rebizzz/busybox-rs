use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct RemoveShellApplet;
impl Applet for RemoveShellApplet {
    fn name(&self) -> &'static str {
        "remove-shell"
    }
    fn description(&self) -> &'static str {
        "Remove a shell from /etc/shells"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        for a in args {
            if is_help(a) {
                return help_out("remove-shell", "SHELL...", "Remove SHELLs from /etc/shells");
            }
        }
        if args.is_empty() {
            eprintln!("Usage: remove-shell SHELL...");
            return Ok(1);
        }
        let shells = read_shells();
        let mut missing = false;
        for a in args {
            let sh = a.as_bytes();
            if !shells.iter().any(|s| s == sh) {
                eprintln!(
                    "remove-shell: '{}' not in {}",
                    String::from_utf8_lossy(sh),
                    shells_path().display()
                );
                missing = true;
            }
        }
        if missing {
            return Ok(1);
        }
        let gone: Vec<&[u8]> = args.iter().map(|a| a.as_bytes()).collect();
        let mut out = Vec::new();
        for s in &shells {
            if !gone.contains(&s.as_slice()) {
                out.extend_from_slice(s);
                out.push(b'\n');
            }
        }
        if let Err(e) = atomic_replace(&shells_path(), &out) {
            eprintln!(
                "remove-shell: cannot update {}: {}",
                shells_path().display(),
                e
            );
            return Ok(1);
        }
        Ok(0)
    }
}
