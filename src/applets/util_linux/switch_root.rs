use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct SwitchRootApplet;
impl Applet for SwitchRootApplet {
    fn name(&self) -> &'static str {
        "switch_root"
    }
    fn description(&self) -> &'static str {
        "Switch to another filesystem as the root"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut console: Option<Vec<u8>> = None;
        let mut pos: Vec<OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-c" {
                i += 1;
                if i >= args.len() {
                    eprintln!("switch_root: -c needs a console");
                    return Ok(1);
                }
                console = Some(args[i].as_bytes().to_vec());
            } else if b.first() == Some(&b'-') {
                eprintln!("switch_root: unknown option");
                return Ok(1);
            } else {
                pos.push(args[i].clone());
            }
            i += 1;
        }
        if pos.len() < 2 {
            eprintln!("usage: switch_root [-c CONSOLE] NEWROOT INIT [ARGS...]");
            return Ok(1);
        }
        use std::ffi::CString;
        let newroot = pos[0].as_bytes().to_vec();

        if !chdir_cstr(&newroot) {
            eprintln!(
                "switch_root: '{}': {}",
                pos[0].to_string_lossy(),
                std::io::Error::last_os_error()
            );
            return Ok(1);
        }
        let dot = CString::new(".").unwrap();
        let slash = CString::new("/").unwrap();
        unsafe {
            if libc::mount(
                dot.as_ptr(),
                slash.as_ptr(),
                std::ptr::null(),
                libc::MS_MOVE,
                std::ptr::null(),
            ) != 0
            {
                eprintln!(
                    "switch_root: mount --move: {}",
                    std::io::Error::last_os_error()
                );
                return Ok(1);
            }
            if libc::chroot(dot.as_ptr()) != 0 {
                eprintln!("switch_root: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            if libc::chdir(slash.as_ptr()) != 0 {
                eprintln!("switch_root: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
        }
        if let Some(c) = console {
            let _ = c;
        }
        let cmd: Vec<OsString> = pos[1..].to_vec();
        Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
    }
}
