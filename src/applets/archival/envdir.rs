use crate::core::{Applet, Result};
use std::ffi::{CString, OsStr, OsString};
use std::io::BufRead;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

fn pw_lookup(name: &str) -> Option<(u32, u32)> {
    if let Ok(n) = name.parse::<u32>() {
        unsafe {
            let p = libc::getpwuid(n);
            if !p.is_null() {
                return Some(((*p).pw_uid, (*p).pw_gid));
            }
            return Some((n, n));
        }
    }
    let c = CString::new(name).ok()?;
    unsafe {
        let p = libc::getpwnam(c.as_ptr());
        if p.is_null() {
            return None;
        }
        Some(((*p).pw_uid, (*p).pw_gid))
    }
}

fn gr_lookup(name: &str) -> Option<u32> {
    if let Ok(n) = name.parse::<u32>() {
        return Some(n);
    }
    let c = CString::new(name).ok()?;
    unsafe {
        let g = libc::getgrnam(c.as_ptr());
        if g.is_null() {
            return None;
        }
        Some((*g).gr_gid)
    }
}

fn split_user_group(s: &str) -> (String, Option<String>) {
    if let Some((u, g)) = s.split_once([':', '.']) {
        (u.to_owned(), Some(g.to_owned()))
    } else {
        (s.to_owned(), None)
    }
}

fn exec_prog(prog: &OsString, args: &[OsString], uid: Option<u32>, gid: Option<u32>) -> i32 {
    let mut cmd = Command::new(prog);
    cmd.args(args);
    if let Some(u) = uid {
        cmd.uid(u);
    }
    if let Some(g) = gid {
        cmd.gid(g);
    }
    let err = cmd.exec();
    eprintln!("{}: {err}", prog.to_string_lossy());
    1
}

pub struct EnvdirApplet;
impl Applet for EnvdirApplet {
    fn name(&self) -> &'static str {
        "envdir"
    }
    fn description(&self) -> &'static str {
        "Set env from dir files then exec prog (daemontools semantics)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("envdir: usage: envdir dir prog...");
            return Ok(1);
        }
        let dir = Path::new(&args[0]);
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(err) => {
                eprintln!("envdir: {}: {err}", dir.display());
                return Ok(1);
            }
        };
        let mut cmd = Command::new(&args[1]);
        cmd.args(&args[2..]);
        for ent in entries.flatten() {
            let fname = ent.file_name();
            let b = fname.as_bytes();
            if b.is_empty() || b.contains(&b'=') {
                continue;
            }
            let Ok(name) = std::str::from_utf8(b) else {
                continue;
            };
            if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                continue;
            }
            let data = match std::fs::read(ent.path()) {
                Ok(d) => d,
                Err(_) => continue,
            };
            if data.is_empty() {
                cmd.env_remove(name);
                continue;
            }

            let mut line = data.split(|&c| c == b'\n').next().unwrap_or(&[]).to_vec();
            for c in line.iter_mut() {
                if *c == 0 {
                    *c = b'\n';
                }
            }
            while line.last().is_some_and(|c| *c == b' ' || *c == b'\t') {
                line.pop();
            }
            cmd.env(name, OsStr::from_bytes(&line));
        }
        let err = cmd.exec();
        eprintln!("envdir: {}: {err}", args[1].to_string_lossy());
        Ok(1)
    }
}
