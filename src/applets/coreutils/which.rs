use crate::core::{Applet, Result};
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct WhichApplet;
impl Applet for WhichApplet {
    fn name(&self) -> &'static str {
        "which"
    }
    fn description(&self) -> &'static str {
        "Locate a command"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            return Ok(1);
        }
        let path_var = env::var_os("PATH").unwrap_or_default();
        let path_bytes = if path_var.is_empty() {
            b"/run/current-system/sw/bin:/bin:/usr/bin:/sbin:/usr/sbin".as_slice()
        } else {
            path_var.as_bytes()
        };
        let paths: Vec<&[u8]> = path_bytes.split(|&b| b == b':').collect();
        let mut ret = 0;

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for cmd in args {
            let cmd_bytes = cmd.as_bytes();
            let mut found = false;
            if cmd_bytes.contains(&b'/') {
                let p = Path::new(cmd);
                if p.is_file() {
                    handle.write_all(p.as_os_str().as_bytes())?;
                    handle.write_all(b"\n")?;
                    found = true;
                }
            } else {
                for dir in &paths {
                    let mut buf = dir.to_vec();
                    buf.push(b'/');
                    buf.extend_from_slice(cmd_bytes);
                    let p = Path::new(std::ffi::OsStr::from_bytes(&buf));
                    if p.is_file() {
                        handle.write_all(p.as_os_str().as_bytes())?;
                        handle.write_all(b"\n")?;
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                ret = 1;
            }
        }
        Ok(ret)
    }
}
