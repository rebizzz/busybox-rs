use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;

pub struct AdduserApplet;
impl Applet for AdduserApplet {
    fn name(&self) -> &'static str {
        "adduser"
    }
    fn description(&self) -> &'static str {
        "Create a user (subset: passwd/shadow/home)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut home: Option<Vec<u8>> = None;
        let mut no_home = false;
        let mut gecos = b"Linux User".to_vec();
        let mut shell = b"/bin/sh".to_vec();
        let mut extra_group: Option<Vec<u8>> = None;
        let mut system = false;
        let mut uid_opt: Option<u32> = None;
        let mut name: Option<Vec<u8>> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "adduser",
                    "[-H] [-S] [-D] [-h DIR] [-g GECOS] [-s SHELL] [-G GRP] [-u UID] USER",
                    "Create a new user",
                );
            } else if b == b"-H" {
                no_home = true;
            } else if b == b"-S" {
                system = true;
            } else if b == b"-D" {
            } else if b == b"-h" || b == b"-g" || b == b"-s" || b == b"-G" || b == b"-u" {
                i += 1;
                if i >= args.len() {
                    eprintln!("adduser: option requires an argument");
                    return Ok(1);
                }
                let v = args[i].as_bytes().to_vec();
                match b {
                    b"-h" => home = Some(v),
                    b"-g" => gecos = v,
                    b"-s" => shell = v,
                    b"-G" => extra_group = Some(v),
                    _ => match std::str::from_utf8(&v)
                        .ok()
                        .and_then(|s| s.parse::<u32>().ok())
                    {
                        Some(n) => uid_opt = Some(n),
                        None => {
                            eprintln!("adduser: invalid UID");
                            return Ok(1);
                        }
                    },
                }
            } else if b.starts_with(b"-") {
                eprintln!("adduser: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else if name.is_none() {
                name = Some(b.to_vec());
            } else {
                eprintln!("adduser: too many arguments");
                return Ok(1);
            }
            i += 1;
        }
        let name = match name {
            Some(n) => n,
            None => {
                eprintln!("Usage: adduser [-H] [-S] [-h DIR] [-g GECOS] [-s SHELL] [-G GRP] [-u UID] USER");
                return Ok(1);
            }
        };
        if name.contains(&b':') || name.contains(&b'/') || name.is_empty() || name.len() > 32 {
            eprintln!("adduser: invalid user name");
            return Ok(1);
        }
        let data = fs::read(passwd_path()).unwrap_or_default();
        let mut ents = parse_passwd(&data);
        if ents.iter().any(|e| e.name == name) {
            eprintln!("adduser: user '{}' exists", String::from_utf8_lossy(&name));
            return Ok(1);
        }
        let used: Vec<u32> = ents.iter().map(|e| e.uid).collect();
        let uid = match uid_opt {
            Some(u) => {
                if used.contains(&u) {
                    eprintln!("adduser: UID {} in use", u);
                    return Ok(1);
                }
                u
            }
            None => next_free_id(&used, system),
        };

        let gdata = fs::read(group_path()).unwrap_or_default();
        let mut glines: Vec<Vec<u8>> = gdata
            .split(|&b| b == b'\n')
            .filter(|l| !l.is_empty())
            .map(|l| l.to_vec())
            .collect();
        let mygroup = String::from_utf8_lossy(&name).into_owned();
        let mut gid = uid;
        let mut have_group = false;
        for g in &glines {
            let f: Vec<&[u8]> = g.split(|&b| b == b':').collect();
            if f.len() >= 3 && f[0] == name.as_slice() {
                if let Ok(n) = std::str::from_utf8(f[2]).unwrap_or("").parse::<u32>() {
                    gid = n;
                    have_group = true;
                }
            }
        }
        if !have_group {
            let gused: Vec<u32> = glines
                .iter()
                .filter_map(|g| {
                    let f: Vec<&[u8]> = g.split(|&b| b == b':').collect();
                    if f.len() >= 3 {
                        std::str::from_utf8(f[2]).ok()?.parse().ok()
                    } else {
                        None
                    }
                })
                .collect();
            if gused.contains(&gid) {
                gid = next_free_id(&gused, system);
            }
            let mut nl = Vec::new();
            nl.extend_from_slice(&name);
            nl.push(b':');
            nl.extend_from_slice(b"x:");
            nl.extend_from_slice(gid.to_string().as_bytes());
            nl.extend_from_slice(b":");
            glines.push(nl);
        }
        if let Some(ref g) = extra_group {
            let mut hit = false;
            for gl in glines.iter_mut() {
                let f: Vec<&[u8]> = gl.split(|&b| b == b':').collect();
                if f.len() >= 4 && f[0] == g.as_slice() {
                    hit = true;
                    let members = f[3];
                    let already = members.split(|&b| b == b',').any(|m| m == name.as_slice());
                    if !already {
                        let mut ng = gl.clone();
                        if !members.is_empty() {
                            ng.push(b',');
                        }
                        ng.extend_from_slice(&name);
                        *gl = ng;
                    }
                }
            }
            if !hit {
                eprintln!("adduser: unknown group '{}'", String::from_utf8_lossy(g));
                return Ok(1);
            }
            let _ = mygroup;
        }
        let hdir = home.unwrap_or_else(|| {
            if no_home {
                Vec::new()
            } else {
                let mut h = b"/home/".to_vec();
                h.extend_from_slice(&name);
                h
            }
        });
        ents.push(PasswdEnt {
            name: name.clone(),
            pass: b"x".to_vec(),
            uid,
            gid,
            gecos,
            home: hdir.clone(),
            shell,
        });
        if let Err(e) = atomic_replace(&passwd_path(), &render_passwd(&ents)) {
            eprintln!("adduser: cannot update passwd: {}", e);
            return Ok(1);
        }

        let mut slines = read_shadow_raw();
        if !slines.iter().any(|f| !f.is_empty() && f[0] == name) {
            slines.push(vec![
                name.clone(),
                b"!".to_vec(),
                b"18000".to_vec(),
                b"0".to_vec(),
                b"99999".to_vec(),
                b"7".to_vec(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ]);
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
                eprintln!("adduser: cannot update shadow: {}", e);
                return Ok(1);
            }
        }
        let mut gout = Vec::new();
        for g in &glines {
            gout.extend_from_slice(g);
            gout.push(b'\n');
        }
        if let Err(e) = atomic_replace(&group_path(), &gout) {
            eprintln!("adduser: cannot update group: {}", e);
            return Ok(1);
        }
        if !hdir.is_empty() && !no_home {
            let hp = std::path::Path::new(std::ffi::OsStr::from_bytes(&hdir));
            if let Err(e) = fs::create_dir_all(hp) {
                eprintln!("adduser: cannot create home: {}", e);
            } else {
                if let Some(c) = cstr(&hdir) {
                    unsafe {
                        libc::chown(c.as_ptr(), uid, gid);
                    }
                }
            }
        }
        Ok(0)
    }
}
