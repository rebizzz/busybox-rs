use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct StartStopDaemonApplet;
impl Applet for StartStopDaemonApplet {
    fn name(&self) -> &'static str {
        "start-stop-daemon"
    }
    fn description(&self) -> &'static str {
        "Start and stop system daemon programs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut start = false;
        let mut stop = false;
        let mut pidfile: Option<Vec<u8>> = None;
        let mut execp: Option<Vec<u8>> = None;
        let mut startas: Option<Vec<u8>> = None;
        let mut name: Option<Vec<u8>> = None;
        let mut signal = libc::SIGTERM;
        let mut background = false;
        let mut make_pidfile = false;
        let mut quiet = false;
        let mut test = false;
        let mut oknodo = false;
        let mut chdir: Option<Vec<u8>> = None;
        let mut extra: Vec<OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--start" || b == b"-S" {
                start = true;
            } else if b == b"--stop" || b == b"-K" {
                stop = true;
            } else if b == b"--background" || b == b"-b" {
                background = true;
            } else if b == b"--make-pidfile" || b == b"-m" {
                make_pidfile = true;
            } else if b == b"--chdir" || b == b"-d" {
                i += 1;
                if i >= args.len() {
                    eprintln!("start-stop-daemon: -d needs an argument");
                    return Ok(1);
                }
                chdir = Some(args[i].as_bytes().to_vec());
            } else if b == b"--quiet" || b == b"-q" {
                quiet = true;
            } else if b == b"--test" || b == b"-t" {
                test = true;
            } else if b == b"--oknodo" || b == b"-o" {
                oknodo = true;
            } else if b == b"--pidfile" || b == b"-p" {
                i += 1;
                if i >= args.len() {
                    eprintln!("start-stop-daemon: --pidfile needs an argument");
                    return Ok(1);
                }
                pidfile = Some(args[i].as_bytes().to_vec());
            } else if b == b"--exec" || b == b"-x" {
                i += 1;
                if i >= args.len() {
                    eprintln!("start-stop-daemon: --exec needs an argument");
                    return Ok(1);
                }
                execp = Some(args[i].as_bytes().to_vec());
            } else if b == b"--startas" || b == b"-a" {
                i += 1;
                if i >= args.len() {
                    eprintln!("start-stop-daemon: --startas needs an argument");
                    return Ok(1);
                }
                startas = Some(args[i].as_bytes().to_vec());
            } else if b == b"--name" || b == b"-n" {
                i += 1;
                if i >= args.len() {
                    eprintln!("start-stop-daemon: --name needs an argument");
                    return Ok(1);
                }
                name = Some(args[i].as_bytes().to_vec());
            } else if b == b"--signal" || b == b"-s" {
                i += 1;
                if i >= args.len() {
                    eprintln!("start-stop-daemon: --signal needs an argument");
                    return Ok(1);
                }
                match sig_from_name(args[i].as_bytes()) {
                    Some(s) => signal = s,
                    None => {
                        eprintln!("start-stop-daemon: unknown signal");
                        return Ok(1);
                    }
                }
            } else if b == b"--" {
                i += 1;
                while i < args.len() {
                    extra.push(args[i].clone());
                    i += 1;
                }
                break;
            } else if b.first() == Some(&b'-') {
                eprintln!("start-stop-daemon: unknown option");
                return Ok(1);
            } else {
                extra.push(args[i].clone());
            }
            i += 1;
        }
        if start == stop {
            eprintln!("start-stop-daemon: need exactly one of --start or --stop");
            return Ok(1);
        }
        if start && execp.is_none() {
            eprintln!("start-stop-daemon: need -x");
            return Ok(1);
        }
        let read_pidfile = || -> Option<i32> {
            let pf = pidfile.as_ref()?;
            let ps = std::str::from_utf8(pf).ok()?;
            let data = std::fs::read(ps).ok()?;
            std::str::from_utf8(&data).ok()?.trim().parse::<i32>().ok()
        };
        let alive = |pid: i32| -> bool {
            if pid <= 0 {
                return false;
            }
            unsafe { libc::kill(pid, 0) == 0 }
        };
        let exec_matches = |pid: i32| -> bool {
            let want_exec = execp.as_ref().or(startas.as_ref());
            let want = match (want_exec, name.as_ref()) {
                (Some(e), _) => {
                    let wb = match e.iter().rposition(|&c| c == b'/') {
                        Some(p) => &e[p + 1..],
                        None => &e[..],
                    };
                    wb.to_vec()
                }
                (None, Some(n)) => n.clone(),
                (None, None) => return true,
            };

            if let Some(st) = read_stat(pid as u32) {
                if st.comm == want {
                    return true;
                }
            }
            let mut cmd = [0u8; 1024];
            let n = read_cmdline(pid as u32, &mut cmd);
            if n > 0 {
                let end = cmd[..n].iter().position(|&c| c == 0).unwrap_or(n);
                let a0 = &cmd[..end];
                let base = match a0.iter().rposition(|&c| c == b'/') {
                    Some(p) => &a0[p + 1..],
                    None => a0,
                };
                if base == want {
                    return true;
                }
            }
            false
        };
        if start {
            if let Some(ref d) = chdir {
                let p = Path::new(std::ffi::OsStr::from_bytes(d));
                if let Err(e) = std::env::set_current_dir(p) {
                    eprintln!("start-stop-daemon: cannot chdir: {}", e);
                    return Ok(1);
                }
            }
            if let Some(pid) = read_pidfile() {
                if alive(pid) && exec_matches(pid) {
                    if !quiet {
                        eprintln!("start-stop-daemon: already running");
                    }
                    return Ok(if oknodo { 0 } else { 1 });
                }
            }
            let bin = match execp {
                Some(p) => p,
                None => {
                    eprintln!("start-stop-daemon: --exec required");
                    return Ok(1);
                }
            };
            let argv0 = startas.unwrap_or_else(|| bin.clone());
            if test {
                return Ok(0);
            }
            let pid = unsafe { libc::fork() };
            if pid < 0 {
                eprintln!("start-stop-daemon: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            if pid == 0 {
                unsafe {
                    libc::setsid();
                    if background {
                        let devnull = libc::open(c"/dev/null".as_ptr(), libc::O_RDWR);
                        if devnull >= 0 {
                            libc::dup2(devnull, 0);
                            libc::dup2(devnull, 1);
                            libc::dup2(devnull, 2);
                            if devnull > 2 {
                                libc::close(devnull);
                            }
                        }
                    }
                }
                let rc = exec_prog_argv0(&bin, &argv0, &extra);
                unsafe { libc::_exit(rc) };
            }
            if make_pidfile {
                if let Some(pf) = &pidfile {
                    if let Ok(ps) = std::str::from_utf8(pf) {
                        let mut s = Vec::with_capacity(16);
                        push_u64(&mut s, pid as u64);
                        s.push(b'\n');
                        let _ = std::fs::write(ps, &s);
                    }
                }
            }
            if !background {
                let mut status = 0;
                unsafe {
                    libc::waitpid(pid, &mut status, 0);
                }
                if libc::WIFEXITED(status) {
                    return Ok(libc::WEXITSTATUS(status));
                } else {
                    return Ok(1);
                }
            }
            return Ok(0);
        }

        let mut targets: Vec<i32> = Vec::new();
        if let Some(pid) = read_pidfile() {
            if alive(pid) && exec_matches(pid) {
                targets.push(pid);
            } else if !oknodo {
                if !quiet {
                    eprintln!("start-stop-daemon: no matching process");
                }
                return Ok(1);
            }
        } else if execp.is_some() || name.is_some() {
            let want = execp
                .as_ref()
                .map(|e| match e.iter().rposition(|&c| c == b'/') {
                    Some(p) => e[p + 1..].to_vec(),
                    None => e.clone(),
                })
                .or_else(|| name.clone())
                .unwrap_or_default();
            for pid in proc_pids() {
                if let Some(st) = read_stat(pid) {
                    if st.comm == want && alive(pid as i32) {
                        targets.push(pid as i32);
                    }
                }
            }
            if targets.is_empty() && !oknodo {
                if !quiet {
                    eprintln!("start-stop-daemon: no matching process");
                }
                return Ok(1);
            }
        } else {
            eprintln!("start-stop-daemon: --pidfile or --exec required for --stop");
            return Ok(1);
        }
        if test {
            return Ok(if targets.is_empty() { 1 } else { 0 });
        }
        let mut rc = 0;
        for pid in targets {
            if unsafe { libc::kill(pid, signal) } != 0 {
                eprintln!(
                    "start-stop-daemon: {}: {}",
                    pid,
                    std::io::Error::last_os_error()
                );
                rc = 1;
            }
        }
        if rc == 0 {
            if let Some(pf) = &pidfile {
                if let Ok(ps) = std::str::from_utf8(pf) {
                    let _ = std::fs::remove_file(ps);
                }
            }
        }
        Ok(rc)
    }
}
