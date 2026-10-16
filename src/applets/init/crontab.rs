#![allow(unused_imports, dead_code, clippy::all)]
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub struct CrontabApplet;
impl Applet for CrontabApplet {
    fn name(&self) -> &'static str {
        "crontab"
    }
    fn description(&self) -> &'static str {
        "Manage user crontabs (subset: -l -r -e, install)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut user: Option<Vec<u8>> = None;
        let mut list = false;
        let mut remove = false;
        let mut edit = false;
        let mut dir = cron_dir();
        let mut file: Option<&OsString> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "crontab",
                    "[-u USER] [-l|-r|-e] [-c DIR] [FILE|-]",
                    "Manage user crontabs",
                );
            } else if b == b"-l" {
                list = true;
            } else if b == b"-r" {
                remove = true;
            } else if b == b"-e" {
                edit = true;
            } else if b == b"-u" {
                i += 1;
                if i >= args.len() {
                    eprintln!("crontab: option requires an argument -- 'u'");
                    return Ok(1);
                }
                user = Some(args[i].as_bytes().to_vec());
            } else if b == b"-c" {
                i += 1;
                if i >= args.len() {
                    eprintln!("crontab: option requires an argument -- 'c'");
                    return Ok(1);
                }
                dir = std::path::PathBuf::from(&args[i]);
            } else if b == b"-" || !b.starts_with(b"-") {
                file = Some(&args[i]);
            } else {
                eprintln!("crontab: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }
        let nmodes = [list, remove, edit].iter().filter(|&&x| x).count();
        if nmodes > 1 {
            eprintln!("crontab: only one of -l, -r, -e");
            return Ok(1);
        }
        let me = current_username();
        let target = user.unwrap_or_else(|| me.clone());

        let euid = unsafe { libc::geteuid() };
        if target != me && euid != 0 {
            eprintln!("crontab: must be privileged to use -u");
            return Ok(1);
        }
        let tname = String::from_utf8_lossy(&target).into_owned();
        if tname.contains('/') || tname.contains('\0') || tname.is_empty() {
            eprintln!("crontab: invalid user");
            return Ok(1);
        }
        let path = dir.join(&tname);
        if list {
            match fs::read(&path) {
                Ok(d) => {
                    let stdout = io::stdout();
                    let mut o = stdout.lock();
                    o.write_all(&d)?;
                    if !d.ends_with(b"\n") {
                        o.write_all(b"\n")?;
                    }
                    Ok(0)
                }
                Err(_) => {
                    eprintln!("crontab: no crontab for {}", tname);
                    Ok(1)
                }
            }
        } else if remove {
            match fs::remove_file(&path) {
                Ok(()) => Ok(0),
                Err(e) => {
                    eprintln!("crontab: cannot remove {}: {}", tname, e);
                    Ok(1)
                }
            }
        } else if edit {
            crontab_edit(&tname, &path)
        } else if let Some(f) = file {
            let data = if f.as_bytes() == b"-" {
                let mut s = Vec::new();
                io::stdin().lock().read_to_end(&mut s)?;
                s
            } else {
                match fs::read(std::path::Path::new(f)) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("crontab: cannot read '{}': {}", f.to_string_lossy(), e);
                        return Ok(1);
                    }
                }
            };

            let (_, errs) = parse_crontab(&data);
            for e in &errs {
                eprintln!(
                    "crontab: warning: ignoring bad line '{}'",
                    String::from_utf8_lossy(e)
                );
            }
            if let Some(parent) = path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("crontab: cannot create dir: {}", e);
                    return Ok(1);
                }
            }
            let tmp = path.with_extension("tmp");
            if let Err(e) = fs::write(&tmp, &data) {
                eprintln!("crontab: cannot install: {}", e);
                return Ok(1);
            }

            let pc = cstr(tmp.as_os_str().as_bytes()).unwrap();
            unsafe {
                libc::chmod(pc.as_ptr(), 0o600);
            }
            if let Err(e) = fs::rename(&tmp, &path) {
                eprintln!("crontab: cannot install: {}", e);
                return Ok(1);
            }
            Ok(0)
        } else {
            eprintln!("Usage: crontab [-u USER] [-l|-r|-e] [FILE|-]");
            Ok(1)
        }
    }
}
