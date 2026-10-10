use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;

applet!(IpcsApplet, "ipcs", "Show IPC facilities", run_ipcs);
fn run_ipcs(args: &[OsString]) -> Result<i32> {
    let (mut m, mut s, mut q) = (false, false, false);
    for a in args {
        let b = ab(a);
        if b == b"-a" || b == b"--all" {
            m = true;
            s = true;
            q = true;
        } else if b == b"-m" {
            m = true;
        } else if b == b"-s" {
            s = true;
        } else if b == b"-q" {
            q = true;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("ipcs: invalid option '{}'", lossy(a));
            return Ok(1);
        }
    }
    if !m && !s && !q {
        m = true;
        s = true;
        q = true;
    }
    let mut out = wlock();
    unsafe {
        if m {
            let _ = writeln!(out, "------ Shared Memory Segments --------");
            let _ = writeln!(out, "{:<8} {:<10} {:<6}", "shmid", "perms", "nattch");
            for id in 0..128 {
                let mut ds: libc::shmid_ds = std::mem::zeroed();
                if libc::shmctl(id, libc::IPC_STAT, &mut ds) == 0 {
                    let _ = writeln!(
                        out,
                        "{:<8} {:<10o} {:<6}",
                        id,
                        ds.shm_perm.mode & 0o777,
                        ds.shm_nattch
                    );
                }
            }
        }
        if s {
            let _ = writeln!(out, "------ Semaphore Arrays --------");
            let _ = writeln!(out, "{:<8} {:<10}", "semid", "perms");
            for id in 0..128 {
                let mut ds: libc::semid_ds = std::mem::zeroed();

                if libc::semctl(id, 0, libc::IPC_STAT, &mut ds) == 0 {
                    let _ = writeln!(out, "{:<8} {:<10o}", id, ds.sem_perm.mode & 0o777);
                }
            }
        }
        if q {
            let _ = writeln!(out, "------ Message Queues --------");
            let _ = writeln!(out, "{:<8} {:<10} {:<8}", "msqid", "perms", "messages");
            for id in 0..128 {
                let mut ds: libc::msqid_ds = std::mem::zeroed();
                if libc::msgctl(id, libc::IPC_STAT, &mut ds) == 0 {
                    let _ = writeln!(
                        out,
                        "{:<8} {:<10o} {:<8}",
                        id,
                        ds.msg_perm.mode & 0o777,
                        ds.msg_qnum
                    );
                }
            }
        }
    }
    Ok(0)
}

fn read_stat() -> Vec<u8> {
    std::fs::read("/proc/stat").unwrap_or_default()
}
fn cpu_line(d: &[u8], prefix: &[u8]) -> Option<Vec<u64>> {
    for line in d.split(|&c| c == b'\n') {
        if line.starts_with(prefix) && line.get(prefix.len()) == Some(&b' ') {
            let mut v = Vec::new();
            for f in line[prefix.len() + 1..].split(|&c| c == b' ' || c == b'\t') {
                if f.is_empty() {
                    continue;
                }
                v.push(String::from_utf8_lossy(f).parse().unwrap_or(0));
            }
            return Some(v);
        }
    }
    None
}
