use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;

pub struct DeluserApplet;
impl Applet for DeluserApplet {
    fn name(&self) -> &'static str {
        "deluser"
    }
    fn description(&self) -> &'static str {
        "Delete a user (subset: --remove-home, group member)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut remove_home = false;
        let mut pos: Vec<Vec<u8>> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if is_help(a) {
                return help_out(
                    "deluser",
                    "[--remove-home] USER [GROUP]",
                    "Delete a user or group membership",
                );
            } else if b == b"--remove-home" || b == b"-r" {
                remove_home = true;
            } else if b.starts_with(b"-") {
                eprintln!("deluser: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else {
                pos.push(b.to_vec());
            }
        }
        if pos.is_empty() || pos.len() > 2 {
            eprintln!("Usage: deluser [--remove-home] USER [GROUP]");
            return Ok(1);
        }
        if pos.len() == 2 {
            let (user, grp) = (&pos[0], &pos[1]);
            let gdata = fs::read(group_path()).unwrap_or_default();
            let mut glines: Vec<Vec<u8>> = gdata
                .split(|&b| b == b'\n')
                .filter(|l| !l.is_empty())
                .map(|l| l.to_vec())
                .collect();
            let mut hit = false;
            for gl in glines.iter_mut() {
                let f: Vec<&[u8]> = gl.split(|&b| b == b':').collect();
                if f.len() >= 4 && f[0] == grp.as_slice() {
                    hit = true;
                    let members: Vec<&[u8]> = f[3]
                        .split(|&b| b == b',')
                        .filter(|m| *m != user.as_slice() && !m.is_empty())
                        .collect();
                    let mut ng = f[0].to_vec();
                    ng.push(b':');
                    ng.extend_from_slice(f[1]);
                    ng.push(b':');
                    ng.extend_from_slice(f[2]);
                    ng.push(b':');
                    ng.extend_from_slice(&members.join(&b','));
                    *gl = ng;
                }
            }
            if !hit {
                eprintln!("deluser: unknown group '{}'", String::from_utf8_lossy(grp));
                return Ok(1);
            }
            let mut out = Vec::new();
            for g in &glines {
                out.extend_from_slice(g);
                out.push(b'\n');
            }
            if let Err(e) = atomic_replace(&group_path(), &out) {
                eprintln!("deluser: cannot update group: {}", e);
                return Ok(1);
            }
            return Ok(0);
        }
        let user = &pos[0];
        let data = fs::read(passwd_path()).unwrap_or_default();
        let ents = parse_passwd(&data);
        let gone: Vec<&PasswdEnt> = ents.iter().filter(|e| &e.name == user).collect();
        if gone.is_empty() {
            eprintln!("deluser: unknown user '{}'", String::from_utf8_lossy(user));
            return Ok(1);
        }
        let home = gone[0].home.clone();
        let keep: Vec<PasswdEnt> = ents.into_iter().filter(|e| &e.name != user).collect();
        if let Err(e) = atomic_replace(&passwd_path(), &render_passwd(&keep)) {
            eprintln!("deluser: cannot update passwd: {}", e);
            return Ok(1);
        }
        let slines: Vec<Vec<Vec<u8>>> = read_shadow_raw()
            .into_iter()
            .filter(|f| f.is_empty() || f[0] != *user)
            .collect();
        let mut out = Vec::new();
        for f in &slines {
            for (k, fld) in f.iter().enumerate() {
                if k > 0 {
                    out.push(b':');
                }
                out.extend_from_slice(fld);
            }
            out.push(b'\n');
        }
        if let Err(e) = atomic_replace(&shadow_path(), &out) {
            eprintln!("deluser: cannot update shadow: {}", e);
            return Ok(1);
        }
        if remove_home && !home.is_empty() {
            let hp = std::path::Path::new(std::ffi::OsStr::from_bytes(&home));
            if let Err(e) = fs::remove_dir_all(hp) {
                eprintln!("deluser: cannot remove home: {}", e);
                return Ok(1);
            }
        }
        Ok(0)
    }
}
