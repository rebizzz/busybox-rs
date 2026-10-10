use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::Path;

pub struct MimApplet;

impl Applet for MimApplet {
    fn name(&self) -> &'static str {
        "mim"
    }
    fn description(&self) -> &'static str {
        "Run executable script from memory"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("mim: file required");
            return Ok(1);
        }

        let script_file = Path::new(&args[0]);
        let data = match fs::read(script_file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("mim: can't read {}: {}", script_file.display(), e);
                return Ok(1);
            }
        };

        let name = CString::new("mim").unwrap();
        let mfd = unsafe { libc::syscall(libc::SYS_memfd_create, name.as_ptr(), 1) };
        if mfd < 0 {
            eprintln!("mim: memfd_create failed");
            return Ok(1);
        }

        let mut file = unsafe { File::from_raw_fd(mfd as i32) };
        if file.write_all(&data).is_err() {
            eprintln!("mim: write to memfd failed");
            return Ok(1);
        }

        let mut argv_c: Vec<CString> = Vec::new();
        argv_c.push(CString::new(args[0].as_bytes()).unwrap_or_default());
        for a in &args[1..] {
            argv_c.push(CString::new(a.as_bytes()).unwrap_or_default());
        }
        let mut argv_ptrs: Vec<*const libc::c_char> = argv_c.iter().map(|c| c.as_ptr()).collect();
        argv_ptrs.push(std::ptr::null());

        let env_c: Vec<CString> = std::env::vars()
            .map(|(k, v)| CString::new(format!("{}={}", k, v)).unwrap())
            .collect();
        let mut env_ptrs: Vec<*const libc::c_char> = env_c.iter().map(|c| c.as_ptr()).collect();
        env_ptrs.push(std::ptr::null());

        unsafe {
            libc::fexecve(file.as_raw_fd(), argv_ptrs.as_ptr(), env_ptrs.as_ptr());
        }

        let proc_fd_path = format!("/proc/self/fd/{}", file.as_raw_fd());
        let path_c = CString::new(proc_fd_path).unwrap();
        unsafe {
            libc::execve(path_c.as_ptr(), argv_ptrs.as_ptr(), env_ptrs.as_ptr());
        }

        let err = io::Error::last_os_error();
        eprintln!("mim: execution failed: {}", err);
        Ok(1)
    }
}
