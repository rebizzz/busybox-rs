use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsStr, OsString};
use std::io::BufRead;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

pub struct ChpstApplet;
impl Applet for ChpstApplet {
    fn name(&self) -> &'static str {
        "chpst"
    }
    fn description(&self) -> &'static str {
        "Change state (-u/-U/-e/-/-/n/limits) then exec prog (runit subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut uid: Option<u32> = None;
        let mut gid: Option<u32> = None;
        let mut set_uidgid_env = false;
        let mut envdir: Option<PathBuf> = None;
        let mut root: Option<PathBuf> = None;
        let mut nice: Option<i32> = None;
        let mut verbose = false;
        let mut limits: Vec<(u32, u64)> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-U" {
                set_uidgid_env = true;
                i += 1;
            } else if b == b"-v" {
                verbose = true;
                i += 1;
            } else if b == b"-u" && i + 1 < args.len() {
                let (u, g) = split_user_group(&args[i + 1].to_string_lossy());
                match pw_lookup(&u) {
                    Some((uu, gg)) => {
                        uid = Some(uu);
                        gid = Some(gg);
                    }
                    None => {
                        eprintln!("chpst: unknown account '{u}'");
                        return Ok(111);
                    }
                }
                if let Some(g) = g {
                    match gr_lookup(&g) {
                        Some(id) => gid = Some(id),
                        None => {
                            eprintln!("chpst: unknown group '{g}'");
                            return Ok(111);
                        }
                    }
                }
                i += 2;
            } else if b == b"-e" && i + 1 < args.len() {
                envdir = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            } else if (b == b"-/" || b == b"--root") && i + 1 < args.len() {
                root = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            } else if b == b"-n" && i + 1 < args.len() {
                nice = String::from_utf8_lossy(ab(&args[i + 1])).parse().ok();
                i += 2;
            } else if b.len() == 2 && b[0] == b'-' && i + 1 < args.len() {
                let r = match b[1] {
                    b'a' => Some(libc::RLIMIT_AS),
                    b'd' => Some(libc::RLIMIT_DATA),
                    b'f' => Some(libc::RLIMIT_FSIZE),
                    b'c' => Some(libc::RLIMIT_CORE),
                    b'l' => Some(libc::RLIMIT_MEMLOCK),
                    b'o' => Some(libc::RLIMIT_NOFILE),
                    b'p' => Some(libc::RLIMIT_NPROC),
                    b'r' => Some(libc::RLIMIT_RSS),
                    b's' => Some(libc::RLIMIT_STACK),
                    b't' => Some(libc::RLIMIT_CPU),
                    b'm' => {
                        let n: u64 = String::from_utf8_lossy(ab(&args[i + 1]))
                            .parse()
                            .unwrap_or(0);
                        limits.push((libc::RLIMIT_DATA, n));
                        limits.push((libc::RLIMIT_STACK, n));
                        limits.push((libc::RLIMIT_RSS, n));
                        i += 2;
                        continue;
                    }
                    _ => None,
                };
                match r {
                    Some(res) => {
                        let n: u64 = String::from_utf8_lossy(ab(&args[i + 1]))
                            .parse()
                            .unwrap_or(0);
                        limits.push((res, n));
                        i += 2;
                    }
                    None => break,
                }
            } else {
                break;
            }
        }
        if i >= args.len() {
            eprintln!("chpst: missing prog");
            return Ok(1);
        }
        let mut cmd = Command::new(&args[i]);
        cmd.args(&args[i + 1..]);
        if let Some(d) = envdir {
            match std::fs::read_dir(&d) {
                Ok(entries) => {
                    for ent in entries.flatten() {
                        let b = ent.file_name();
                        let bb = b.as_bytes();
                        if bb.is_empty() || bb.contains(&b'=') {
                            continue;
                        }
                        let Ok(name) = std::str::from_utf8(bb) else {
                            continue;
                        };
                        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                            continue;
                        }
                        match std::fs::read(ent.path()) {
                            Ok(data) if !data.is_empty() => {
                                let mut line =
                                    data.split(|&c| c == b'\n').next().unwrap_or(&[]).to_vec();
                                while line.last().is_some_and(|c| *c == b' ' || *c == b'\t') {
                                    line.pop();
                                }
                                cmd.env(name, OsStr::from_bytes(&line));
                            }
                            Ok(_) => {
                                cmd.env_remove(name);
                            }
                            Err(_) => {}
                        }
                    }
                }
                Err(e) => {
                    eprintln!("chpst: {}: {e}", d.display());
                    return Ok(111);
                }
            }
        }
        if set_uidgid_env {
            if let (Some(u), Some(g)) = (uid, gid) {
                cmd.env("UID", u.to_string());
                cmd.env("GID", g.to_string());
            }
        }

        if root.is_some() || nice.is_some() {
            unsafe {
                cmd.pre_exec(move || {
                    if let Some(ref r) = root {
                        let bytes = r.as_os_str().as_bytes();
                        let c = CString::new(bytes).map_err(|_| {
                            std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad root")
                        })?;
                        if libc::chroot(c.as_ptr()) != 0 {
                            return Err(std::io::Error::last_os_error());
                        }
                        if libc::chdir(c"/".as_ptr()) != 0 {
                            return Err(std::io::Error::last_os_error());
                        }
                    }
                    if let Some(n) = nice {
                        if libc::nice(n) == -1 {
                            return Err(std::io::Error::last_os_error());
                        }
                    }
                    Ok(())
                });
            }
        }
        for (r, n) in &limits {
            if let Err(e) = apply_rlimit(*r, *n) {
                eprintln!("chpst: setrlimit: {e}");
                return Ok(1);
            }
        }
        if let Some(u) = uid {
            cmd.uid(u);
        }
        if let Some(g) = gid {
            cmd.gid(g);
        }
        if verbose {
            eprintln!("chpst: executing {}", args[i].to_string_lossy());
        }
        let err = cmd.exec();
        eprintln!("chpst: {}: {err}", args[i].to_string_lossy());
        Ok(1)
    }
}
