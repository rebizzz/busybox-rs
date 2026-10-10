use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;

pub struct AddgroupApplet;
impl Applet for AddgroupApplet {
    fn name(&self) -> &'static str {
        "addgroup"
    }
    fn description(&self) -> &'static str {
        "Create a group or add a user to one (subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut gid_opt: Option<u32> = None;
        let mut system = false;
        let mut pos: Vec<Vec<u8>> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if is_help(&args[i]) {
                return help_out(
                    "addgroup",
                    "[-g GID] [-S] GROUP | USER GROUP",
                    "Create a group or add a member",
                );
            } else if b == b"-S" {
                system = true;
            } else if b == b"-g" {
                i += 1;
                if i >= args.len() {
                    eprintln!("addgroup: option requires an argument -- 'g'");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    Some(n) => gid_opt = Some(n),
                    None => {
                        eprintln!("addgroup: invalid GID");
                        return Ok(1);
                    }
                }
            } else if b.starts_with(b"-") {
                eprintln!("addgroup: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else {
                pos.push(b.to_vec());
            }
            i += 1;
        }
        let gdata = fs::read(group_path()).unwrap_or_default();
        let mut glines: Vec<Vec<u8>> = gdata
            .split(|&b| b == b'\n')
            .filter(|l| !l.is_empty())
            .map(|l| l.to_vec())
            .collect();
        if pos.len() == 2 {
            let (user, grp) = (&pos[0], &pos[1]);
            if lookup_user(user).is_none() {
                eprintln!("addgroup: unknown user '{}'", String::from_utf8_lossy(user));
                return Ok(1);
            }
            let mut hit = false;
            for gl in glines.iter_mut() {
                let f: Vec<&[u8]> = gl.split(|&b| b == b':').collect();
                if f.len() >= 4 && f[0] == grp.as_slice() {
                    hit = true;
                    if !f[3].split(|&b| b == b',').any(|m| m == user.as_slice()) {
                        let mut ng = gl.clone();
                        if !f[3].is_empty() {
                            ng.push(b',');
                        }
                        ng.extend_from_slice(user);
                        *gl = ng;
                    }
                }
            }
            if !hit {
                eprintln!("addgroup: unknown group '{}'", String::from_utf8_lossy(grp));
                return Ok(1);
            }
        } else if pos.len() == 1 {
            let grp = &pos[0];
            if grp.contains(&b':') || grp.contains(&b'/') || grp.is_empty() {
                eprintln!("addgroup: invalid group name");
                return Ok(1);
            }
            if glines
                .iter()
                .any(|g| g.split(|&b| b == b':').next() == Some(grp.as_slice()))
            {
                eprintln!("addgroup: group '{}' exists", String::from_utf8_lossy(grp));
                return Ok(1);
            }
            let used: Vec<u32> = glines
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
            let gid = match gid_opt {
                Some(g) => {
                    if used.contains(&g) {
                        eprintln!("addgroup: GID {} in use", g);
                        return Ok(1);
                    }
                    g
                }
                None => next_free_id(&used, system),
            };
            let mut nl = grp.clone();
            nl.extend_from_slice(b":x:");
            nl.extend_from_slice(gid.to_string().as_bytes());
            nl.extend_from_slice(b":");
            glines.push(nl);
        } else {
            eprintln!("Usage: addgroup [-g GID] [-S] GROUP | USER GROUP");
            return Ok(1);
        }
        let mut out = Vec::new();
        for g in &glines {
            out.extend_from_slice(g);
            out.push(b'\n');
        }
        if let Err(e) = atomic_replace(&group_path(), &out) {
            eprintln!("addgroup: cannot update group: {}", e);
            return Ok(1);
        }
        Ok(0)
    }
}
