use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct FgconsoleApplet;

impl Applet for FgconsoleApplet {
    fn name(&self) -> &'static str {
        "fgconsole"
    }

    fn description(&self) -> &'static str {
        "Print the number of the active virtual console"
    }

    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("fgconsole: {}", e);
                return Ok(1);
            }
        };

        let mut vt = VtStat {
            v_active: 0,
            v_signal: 0,
            v_state: 0,
        };
        let ret = unsafe {
            libc::ioctl(
                console.as_raw_fd(),
                VT_GETSTATE,
                &mut vt as *mut VtStat as *mut libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!(
                "fgconsole: VT_GETSTATE failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        println!("{}", vt.v_active);
        Ok(0)
    }
}
