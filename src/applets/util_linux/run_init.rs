use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct RunInitApplet;
impl Applet for RunInitApplet {
    fn name(&self) -> &'static str {
        "run-init"
    }
    fn description(&self) -> &'static str {
        "Run init from a new root filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut console: Option<Vec<u8>> = None;
        let mut pos: Vec<OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-d" {
                i += 1;
                if i >= args.len() {
                    eprintln!("run-init: -d needs a console");
                    return Ok(1);
                }
                console = Some(args[i].as_bytes().to_vec());
            } else if b.first() == Some(&b'-') {
                eprintln!("run-init: unknown option");
                return Ok(1);
            } else {
                pos.push(args[i].clone());
            }
            i += 1;
        }
        if pos.len() < 2 {
            eprintln!("usage: run-init [-d CONSOLE] NEWROOT INIT [ARGS...]");
            return Ok(1);
        }
        use std::ffi::CString;
        if !chdir_cstr(pos[0].as_bytes()) {
            eprintln!(
                "run-init: '{}': {}",
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
                    "run-init: mount --move: {}",
                    std::io::Error::last_os_error()
                );
                return Ok(1);
            }
            if libc::chroot(dot.as_ptr()) != 0 {
                eprintln!("run-init: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
        }
        if let Some(c) = console {
            use std::ffi::CString as CS;
            if let Ok(cp) = CS::new(c) {
                unsafe {
                    let fd = libc::open(cp.as_ptr(), libc::O_RDWR);
                    if fd >= 0 {
                        libc::dup2(fd, 0);
                        libc::dup2(fd, 1);
                        libc::dup2(fd, 2);
                        if fd > 2 {
                            libc::close(fd);
                        }
                    }
                }
            }
        }
        let cmd: Vec<OsString> = pos[1..].to_vec();
        Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
    }
}
