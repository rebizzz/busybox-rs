use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct DumpkmapApplet;

impl Applet for DumpkmapApplet {
    fn name(&self) -> &'static str {
        "dumpkmap"
    }

    fn description(&self) -> &'static str {
        "Dump keyboard translation table to standard output"
    }

    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("dumpkmap: {}", e);
                return Ok(1);
            }
        };

        let mut stdout = io::stdout().lock();

        let _ = stdout.write_all(b"bkeymap\0");

        let max_tables: u8 = 7;
        let mut flags = [0u8; 256];
        for (i, item) in flags.iter_mut().enumerate().take(max_tables as usize) {
            *item = 1 << i;
        }
        let _ = stdout.write_all(&flags);

        for table in 0..max_tables {
            for keycode in 0..128u8 {
                let mut ent = Kbentry {
                    kb_table: table,
                    kb_index: keycode,
                    kb_value: 0,
                };
                let ret = unsafe {
                    libc::ioctl(
                        console.as_raw_fd(),
                        KDGKBENT,
                        &mut ent as *mut Kbentry as *mut libc::c_void,
                    )
                };
                let val: u16 = if ret == 0 { ent.kb_value } else { 0 };
                let _ = stdout.write_all(&val.to_le_bytes());
            }
        }
        let _ = stdout.flush();
        Ok(0)
    }
}
