use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;

pub struct PasswdApplet;
impl Applet for PasswdApplet {
    fn name(&self) -> &'static str {
        "passwd"
    }
    fn description(&self) -> &'static str {
        "Change passwords (subset: -l -u -d real; change needs crypt)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut lock = false;
        let mut unlock = false;
        let mut delete = false;
        let mut user: Option<Vec<u8>> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "passwd",
                    "[-a ALG] [-l|-u|-d] [USER]",
                    "Lock/unlock/change passwords",
                );
            } else if b == b"-a" {
                i += 1;
                if i >= args.len() {
                    eprintln!("passwd: option requires an argument -- 'a'");
                    return Ok(1);
                }
            } else if b == b"-l" || b == b"--lock" {
                lock = true;
            } else if b == b"-u" || b == b"--unlock" {
                unlock = true;
            } else if b == b"-d" || b == b"--delete" {
                delete = true;
            } else if b.starts_with(b"-") {
                eprintln!("passwd: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else if user.is_none() {
                user = Some(b.to_vec());
            } else {
                eprintln!("passwd: too many arguments");
                return Ok(1);
            }
            i += 1;
        }
        let nmodes = [lock, unlock, delete].iter().filter(|&&x| x).count();
        if nmodes > 1 {
            eprintln!("passwd: only one of -l, -u, -d");
            return Ok(1);
        }

        let euid = unsafe { libc::geteuid() };
        let me = lookup_uid(unsafe { libc::getuid() })
            .map(|e| e.name)
            .unwrap_or_else(|| b"root".to_vec());
        let target = user.unwrap_or_else(|| me.clone());
        if target != me && euid != 0 {
            eprintln!("passwd: must be privileged to change others");
            return Ok(1);
        }
        if lookup_user(&target).is_none() {
            eprintln!(
                "passwd: unknown user '{}'",
                String::from_utf8_lossy(&target)
            );
            return Ok(1);
        }
        if lock || unlock || delete {
            let mut lines = read_shadow_raw();
            let has_shadow = !lines.is_empty();
            let mut found = false;
            for f in lines.iter_mut() {
                if f.len() >= 2 && f[0] == target {
                    found = true;
                    if lock {
                        if !f[1].starts_with(b"!") {
                            let mut nh = Vec::with_capacity(f[1].len() + 1);
                            nh.push(b'!');
                            nh.extend_from_slice(&f[1]);
                            f[1] = nh;
                        }
                    } else if unlock {
                        if f[1].starts_with(b"!") {
                            f[1] = f[1][1..].to_vec();
                        }
                    } else {
                        f[1].clear();
                    }
                }
            }
            if !found {
                if has_shadow {
                    eprintln!(
                        "passwd: no shadow entry for '{}'",
                        String::from_utf8_lossy(&target)
                    );
                    return Ok(1);
                }

                let data = fs::read(passwd_path()).unwrap_or_default();
                let mut ents = parse_passwd(&data);
                let mut hit = false;
                for e in ents.iter_mut() {
                    if e.name == target {
                        hit = true;
                        if lock {
                            if !e.pass.starts_with(b"!") {
                                let mut nh = vec![b'!'];
                                nh.extend_from_slice(&e.pass);
                                e.pass = nh;
                            }
                        } else if unlock {
                            if e.pass.starts_with(b"!") {
                                e.pass = e.pass[1..].to_vec();
                            }
                        } else {
                            e.pass.clear();
                        }
                    }
                }
                if !hit {
                    eprintln!(
                        "passwd: unknown user '{}'",
                        String::from_utf8_lossy(&target)
                    );
                    return Ok(1);
                }
                if let Err(e) = atomic_replace(&passwd_path(), &render_passwd(&ents)) {
                    eprintln!("passwd: cannot update passwd: {}", e);
                    return Ok(1);
                }
                return Ok(0);
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
                eprintln!("passwd: cannot update shadow: {}", e);
                return Ok(1);
            }
            return Ok(0);
        }

        let p1 = match read_secret(b"New password: ", false) {
            Some(p) => p,
            None => {
                eprintln!("passwd: input error");
                return Ok(1);
            }
        };
        let p2 = match read_secret(b"Retype password: ", false) {
            Some(p) => p,
            None => {
                eprintln!("passwd: input error");
                return Ok(1);
            }
        };
        if p1 != p2 {
            eprintln!("passwd: passwords do not match");
            return Ok(1);
        }
        no_crypt("passwd")
    }
}
