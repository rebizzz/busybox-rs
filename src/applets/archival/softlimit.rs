use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

fn apply_rlimit(res: u32, n: u64) -> std::io::Result<()> {
    let mut cur: libc::rlimit = unsafe { std::mem::zeroed() };
    if unsafe { libc::getrlimit(res, &mut cur) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let max = if cur.rlim_max == libc::RLIM_INFINITY || n < cur.rlim_max {
        n
    } else {
        cur.rlim_max
    };
    let _ = max;
    let new = libc::rlimit {
        rlim_cur: n.min(if cur.rlim_max == libc::RLIM_INFINITY {
            n
        } else {
            cur.rlim_max
        }),
        rlim_max: cur.rlim_max,
    };
    if unsafe { libc::setrlimit(res, &new) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

pub struct SoftlimitApplet;
impl Applet for SoftlimitApplet {
    fn name(&self) -> &'static str {
        "softlimit"
    }
    fn description(&self) -> &'static str {
        "Set soft rlimits (-a/-d/-m/-o/-s/-t/... ) then exec prog"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut limits: Vec<(u32, u64)> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            let flag = if b.len() == 2 && b[0] == b'-' {
                Some(b[1])
            } else {
                None
            };
            let res = match flag {
                Some(b'a') => Some(libc::RLIMIT_AS),
                Some(b'c') => Some(libc::RLIMIT_CORE),
                Some(b'd') => Some(libc::RLIMIT_DATA),
                Some(b'f') => Some(libc::RLIMIT_FSIZE),
                Some(b'l') => Some(libc::RLIMIT_MEMLOCK),
                Some(b'o') => Some(libc::RLIMIT_NOFILE),
                Some(b'p') => Some(libc::RLIMIT_NPROC),
                Some(b'r') => Some(libc::RLIMIT_RSS),
                Some(b's') => Some(libc::RLIMIT_STACK),
                Some(b't') => Some(libc::RLIMIT_CPU),
                Some(b'm') => None,
                _ => None,
            };
            if flag == Some(b'm') && i + 1 < args.len() {
                let n: u64 = String::from_utf8_lossy(ab(&args[i + 1]))
                    .parse()
                    .unwrap_or(0);
                limits.push((libc::RLIMIT_DATA, n));
                limits.push((libc::RLIMIT_STACK, n));
                limits.push((libc::RLIMIT_RSS, n));
                i += 2;
                continue;
            }
            if let Some(r) = res {
                if i + 1 >= args.len() {
                    eprintln!("softlimit: -{} needs a value", flag.unwrap_or(b'?') as char);
                    return Ok(1);
                }
                let n: u64 = String::from_utf8_lossy(ab(&args[i + 1]))
                    .parse()
                    .unwrap_or(0);
                limits.push((r, n));
                i += 2;
                continue;
            }
            break;
        }
        if i >= args.len() {
            eprintln!("softlimit: missing prog");
            return Ok(1);
        }
        for (r, n) in &limits {
            if let Err(e) = apply_rlimit(*r, *n) {
                eprintln!("softlimit: setrlimit: {e}");
                return Ok(1);
            }
        }
        Ok(exec_prog(&args[i], &args[i + 1..], None, None))
    }
}
