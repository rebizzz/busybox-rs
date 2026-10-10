use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;

pub struct ChpasswdApplet;
impl Applet for ChpasswdApplet {
    fn name(&self) -> &'static str {
        "chpasswd"
    }
    fn description(&self) -> &'static str {
        "Bulk password update (subset: -e passthrough real)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut encrypted = false;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "chpasswd",
                    "[-e] [-m] [-c ALG] [-R DIR]",
                    "Update passwords from USER:PASS lines on stdin",
                );
            } else if b == b"-e" || b == b"--encrypted" {
                encrypted = true;
            } else if b == b"-m" {
            } else if b == b"-c" {
                i += 1;
                if i >= args.len() {
                    eprintln!("chpasswd: option requires an argument -- 'c'");
                    return Ok(1);
                }
            } else if b == b"-R" {
                i += 1;
                if i >= args.len() {
                    eprintln!("chpasswd: option requires an argument -- 'R'");
                    return Ok(1);
                }
                match cstr(args[i].as_bytes()) {
                    Some(d) => {
                        if unsafe { libc::chroot(d.as_ptr()) } != 0 {
                            eprintln!("chpasswd: chroot: {}", io::Error::last_os_error());
                            return Ok(1);
                        }
                    }
                    None => {
                        eprintln!("chpasswd: bad directory");
                        return Ok(1);
                    }
                }
            } else {
                eprintln!("chpasswd: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }
        let mut input = Vec::new();
        io::stdin().lock().read_to_end(&mut input)?;
        let mut pairs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        for line in input.split(|&b| b == b'\n') {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if line.is_empty() {
                continue;
            }
            let mut sp = line.splitn(2, |&b| b == b':');
            let (u, p) = (sp.next().unwrap_or(b""), sp.next());
            let p = match p {
                Some(p) => p,
                None => {
                    eprintln!("chpasswd: bad line '{}'", String::from_utf8_lossy(line));
                    return Ok(1);
                }
            };
            if u.is_empty() {
                eprintln!("chpasswd: bad line '{}'", String::from_utf8_lossy(line));
                return Ok(1);
            }
            if p.is_empty() && !encrypted {
                eprintln!(
                    "chpasswd: empty password for '{}'",
                    String::from_utf8_lossy(u)
                );
                return Ok(1);
            }
            if lookup_user(u).is_none() {
                eprintln!("chpasswd: unknown user '{}'", String::from_utf8_lossy(u));
                return Ok(1);
            }
            pairs.push((u.to_vec(), p.to_vec()));
        }
        if !encrypted {
            return no_crypt("chpasswd");
        }
        let mut lines = read_shadow_raw();
        for (u, h) in &pairs {
            let mut hit = false;
            for f in lines.iter_mut() {
                if f.len() >= 2 && f[0] == *u {
                    f[1] = h.clone();
                    hit = true;
                }
            }
            if !hit {
                lines.push(vec![
                    u.clone(),
                    h.clone(),
                    b"18000".to_vec(),
                    b"0".to_vec(),
                    b"99999".to_vec(),
                    b"7".to_vec(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ]);
            }
        }
        let mut out = Vec::new();
        for f in &lines {
            for (k, fld) in f.iter().enumerate() {
                if k > 0 {
                    out.push(b':');
                }
                out.extend_from_slice(fld);
            }
            out.push(b'\n');
        }
        if let Err(e) = atomic_replace(&shadow_path(), &out) {
            eprintln!("chpasswd: cannot update shadow: {}", e);
            return Ok(1);
        }
        Ok(0)
    }
}
