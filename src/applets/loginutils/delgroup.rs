use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;

pub struct DelgroupApplet;
impl Applet for DelgroupApplet {
    fn name(&self) -> &'static str {
        "delgroup"
    }
    fn description(&self) -> &'static str {
        "Delete a group or remove a member (subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut pos: Vec<Vec<u8>> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if is_help(a) {
                return help_out(
                    "delgroup",
                    "GROUP | USER GROUP",
                    "Delete a group or a member",
                );
            } else if b.starts_with(b"-") {
                eprintln!("delgroup: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            } else {
                pos.push(b.to_vec());
            }
        }
        if pos.is_empty() || pos.len() > 2 {
            eprintln!("Usage: delgroup GROUP | USER GROUP");
            return Ok(1);
        }
        let gdata = fs::read(group_path()).unwrap_or_default();
        let glines: Vec<Vec<u8>> = gdata
            .split(|&b| b == b'\n')
            .filter(|l| !l.is_empty())
            .map(|l| l.to_vec())
            .collect();
        let mut out: Vec<Vec<u8>> = Vec::new();
        let mut hit = false;
        if pos.len() == 1 {
            for g in &glines {
                if g.split(|&b| b == b':').next() == Some(pos[0].as_slice()) {
                    hit = true;
                } else {
                    out.push(g.clone());
                }
            }
            if !hit {
                eprintln!(
                    "delgroup: unknown group '{}'",
                    String::from_utf8_lossy(&pos[0])
                );
                return Ok(1);
            }
        } else {
            let (user, grp) = (&pos[0], &pos[1]);
            for g in &glines {
                let f: Vec<&[u8]> = g.split(|&b| b == b':').collect();
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
                    out.push(ng);
                } else {
                    out.push(g.clone());
                }
            }
            if !hit {
                eprintln!("delgroup: unknown group '{}'", String::from_utf8_lossy(grp));
                return Ok(1);
            }
        }
        let mut data = Vec::new();
        for g in &out {
            data.extend_from_slice(g);
            data.push(b'\n');
        }
        if let Err(e) = atomic_replace(&group_path(), &data) {
            eprintln!("delgroup: cannot update group: {}", e);
            return Ok(1);
        }
        Ok(0)
    }
}
