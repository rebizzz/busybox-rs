use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

pub struct IdApplet;
impl Applet for IdApplet {
    fn name(&self) -> &'static str {
        "id"
    }
    fn description(&self) -> &'static str {
        "Print user and group IDs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut u_only = false;
        let mut g_only = false;
        let mut name_mode = false;
        for a in args {
            let b = a.as_bytes();
            if b.first() == Some(&b'-') && b.len() > 1 {
                for &c in &b[1..] {
                    match c {
                        b'u' => u_only = true,
                        b'g' => g_only = true,
                        b'n' => name_mode = true,
                        _ => {}
                    }
                }
            } else {
                eprintln!("id: only the current user is supported in this subset");
                return Ok(1);
            }
        }
        let (uid, gid) = unsafe { (libc::getuid(), libc::getgid()) };
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line: Vec<u8> = Vec::with_capacity(64);
        if u_only {
            if name_mode {
                match pw_name(uid) {
                    Some(n) => line.extend_from_slice(&n),
                    None => push_u64(&mut line, uid as u64),
                }
            } else {
                push_u64(&mut line, uid as u64);
            }
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            return Ok(0);
        }
        if g_only {
            if name_mode {
                match gr_name(gid) {
                    Some(n) => line.extend_from_slice(&n),
                    None => push_u64(&mut line, gid as u64),
                }
            } else {
                push_u64(&mut line, gid as u64);
            }
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            return Ok(0);
        }

        line.extend_from_slice(b"uid=");
        push_u64(&mut line, uid as u64);
        line.push(b'(');
        line.extend_from_slice(&pw_name(uid).unwrap_or_else(|| uid.to_string().into_bytes()));
        line.push(b')');
        line.extend_from_slice(b" gid=");
        push_u64(&mut line, gid as u64);
        line.push(b'(');
        line.extend_from_slice(&gr_name(gid).unwrap_or_else(|| gid.to_string().into_bytes()));
        line.push(b')');

        let mut gids = [0 as libc::gid_t; 32];
        let ng = unsafe { libc::getgroups(gids.len() as i32, gids.as_mut_ptr()) };
        if ng > 0 {
            line.extend_from_slice(b" groups=");
            for (k, &g) in gids[..ng as usize].iter().enumerate() {
                if k > 0 {
                    line.push(b',');
                }
                push_u64(&mut line, g as u64);
                line.push(b'(');
                line.extend_from_slice(&gr_name(g).unwrap_or_else(|| g.to_string().into_bytes()));
                line.push(b')');
            }
        }
        line.push(b'\n');
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}
