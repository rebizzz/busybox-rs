use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

pub struct GroupsApplet;
impl Applet for GroupsApplet {
    fn name(&self) -> &'static str {
        "groups"
    }
    fn description(&self) -> &'static str {
        "Print the groups a user is in"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line: Vec<u8> = Vec::with_capacity(64);
        if args.is_empty() {
            let mut gids = [0 as libc::gid_t; 32];
            let ng = unsafe { libc::getgroups(gids.len() as i32, gids.as_mut_ptr()) };
            if ng <= 0 {
                let gid = unsafe { libc::getgid() };
                match gr_name(gid) {
                    Some(n) => line.extend_from_slice(&n),
                    None => push_u64(&mut line, gid as u64),
                }
            } else {
                for (k, &g) in gids[..ng as usize].iter().enumerate() {
                    if k > 0 {
                        line.push(b' ');
                    }
                    match gr_name(g) {
                        Some(n) => line.extend_from_slice(&n),
                        None => push_u64(&mut line, g as u64),
                    }
                }
            }
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            return Ok(0);
        }

        let mut rc = 0;
        for a in args {
            let name = a.as_bytes();

            let mut nb = [0u8; 65];
            if name.len() >= nb.len() {
                eprintln!("groups: name too long");
                rc = 1;
                continue;
            }
            nb[..name.len()].copy_from_slice(name);
            let (gid, ngroups) = unsafe {
                let pw = libc::getpwnam(nb.as_ptr() as *const libc::c_char);
                if pw.is_null() {
                    (0, -1)
                } else {
                    let g = (*pw).pw_gid;
                    let mut gs = [0 as libc::gid_t; 32];
                    let mut n: i32 = gs.len() as i32;
                    let r = libc::getgrouplist((*pw).pw_name, g, gs.as_mut_ptr(), &mut n);
                    if r < 0 {
                        (g, -1)
                    } else {
                        let _ = &gs;
                        (g, n)
                    }
                }
            };
            if ngroups < 0 {
                eprintln!("groups: '{}': no such user", a.to_string_lossy());
                rc = 1;
                continue;
            }

            let mut gs = [0 as libc::gid_t; 32];
            let mut n: i32 = gs.len() as i32;
            let ok = unsafe {
                libc::getgrouplist(
                    nb.as_ptr() as *const libc::c_char,
                    gid,
                    gs.as_mut_ptr(),
                    &mut n,
                ) >= 0
            };
            line.clear();
            if ok {
                let count = (n as usize).min(gs.len());
                for (k, &g) in gs[..count].iter().enumerate() {
                    if k > 0 {
                        line.push(b' ');
                    }
                    match gr_name(g) {
                        Some(gn) => line.extend_from_slice(&gn),
                        None => push_u64(&mut line, g as u64),
                    }
                }
            } else {
                match gr_name(gid) {
                    Some(gn) => line.extend_from_slice(&gn),
                    None => push_u64(&mut line, gid as u64),
                }
            }
            line.push(b'\n');
            out.write_all(&line)?;
        }
        out.flush()?;
        Ok(rc)
    }
}
