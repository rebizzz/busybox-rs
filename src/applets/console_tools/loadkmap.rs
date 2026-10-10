use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct LoadkmapApplet;

impl Applet for LoadkmapApplet {
    fn name(&self) -> &'static str {
        "loadkmap"
    }

    fn description(&self) -> &'static str {
        "Load keyboard translation table from standard input"
    }

    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("loadkmap: {}", e);
                return Ok(1);
            }
        };

        let mut stdin = io::stdin().lock();
        let mut magic = [0u8; 8];
        if stdin.read_exact(&mut magic).is_err() || &magic != b"bkeymap\0" {
            eprintln!("loadkmap: bad magic header");
            return Ok(1);
        }

        let mut flags = [0u8; 256];
        if stdin.read_exact(&mut flags).is_err() {
            eprintln!("loadkmap: truncated keymap flags");
            return Ok(1);
        }

        for (table, &flag) in flags.iter().enumerate() {
            if flag == 0 {
                continue;
            }
            for keycode in 0..128u8 {
                let mut buf = [0u8; 2];
                if stdin.read_exact(&mut buf).is_err() {
                    break;
                }
                let val = u16::from_le_bytes(buf);
                let mut ent = Kbentry {
                    kb_table: table as u8,
                    kb_index: keycode,
                    kb_value: val,
                };
                unsafe {
                    libc::ioctl(
                        console.as_raw_fd(),
                        KDSKBENT,
                        &mut ent as *mut Kbentry as *mut libc::c_void,
                    );
                }
            }
        }
        Ok(0)
    }
}
