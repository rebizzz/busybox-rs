#![allow(unused_imports, dead_code, clippy::all)]
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::io::FromRawFd;

pub fn is_help(a: &OsString) -> bool {
    let b = a.as_bytes();
    b == b"--help"
}

pub fn help_out(name: &str, usage: &str, desc: &str) -> Result<i32> {
    let stdout = io::stdout();
    let mut o = stdout.lock();
    o.write_all(b"Usage: ")?;
    o.write_all(name.as_bytes())?;
    o.write_all(b" ")?;
    o.write_all(usage.as_bytes())?;
    o.write_all(b"\n")?;
    o.write_all(desc.as_bytes())?;
    o.write_all(b"\n")?;
    Ok(0)
}

pub fn cstr(bytes: &[u8]) -> Option<CString> {
    CString::new(bytes).ok()
}

pub fn fill_c_field(dst: &mut [libc::c_char], src: &[u8]) {
    let n = dst.len().min(src.len());
    for (i, &b) in src[..n].iter().enumerate() {
        dst[i] = b as libc::c_char;
    }

    if n < dst.len() {
        dst[n] = 0;
    }
}

pub fn cfield_to_vec(src: &[libc::c_char]) -> Vec<u8> {
    let raw: Vec<u8> = src.iter().map(|&c| c as u8).collect();
    match raw.iter().position(|&b| b == 0) {
        Some(i) => raw[..i].to_vec(),
        None => raw,
    }
}

#[derive(Clone, Default)]
pub struct UtEntry {
    pub typ: i16,
    pub pid: i32,
    pub line: Vec<u8>,
    pub user: Vec<u8>,
    pub host: Vec<u8>,
    pub tv_sec: i64,
}

pub const UT_RUN_LVL: i16 = 1;
pub const UT_BOOT_TIME: i16 = 2;
pub const UT_USER_PROCESS: i16 = 7;
pub const UT_DEAD_PROCESS: i16 = 8;

pub fn read_utmpx(override_path: Option<&std::path::Path>) -> Vec<UtEntry> {
    let mut out = Vec::new();

    unsafe {
        let _path_guard;
        if let Some(p) = override_path {
            match cstr(p.as_os_str().as_bytes()) {
                Some(c) => {
                    _path_guard = c;
                    libc::utmpxname(_path_guard.as_ptr());
                }
                None => return out,
            }
        }
        libc::setutxent();
        loop {
            let p = libc::getutxent();
            if p.is_null() {
                break;
            }
            let u: libc::utmpx = std::ptr::read(p);
            out.push(UtEntry {
                typ: u.ut_type as i16,
                pid: u.ut_pid,
                line: cfield_to_vec(&u.ut_line),
                user: cfield_to_vec(&u.ut_user),
                host: cfield_to_vec(&u.ut_host),
                tv_sec: u.ut_tv.tv_sec as i64,
            });
        }
        libc::endutxent();
    }
    out
}

pub fn write_utmpx(
    path: &std::path::Path,
    typ: i16,
    pid: i32,
    line: &[u8],
    id: &[u8],
    user: &[u8],
) -> bool {
    let p = cstr(path.as_os_str().as_bytes());
    let p = match p {
        Some(c) => c,
        None => return false,
    };
    if !path.exists()
        && OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .is_err()
    {
        return false;
    }

    unsafe {
        let mut u: libc::utmpx = std::mem::zeroed();
        u.ut_type = typ as libc::c_short;
        u.ut_pid = pid;
        fill_c_field(&mut u.ut_line, line);
        fill_c_field(&mut u.ut_id, id);
        fill_c_field(&mut u.ut_user, user);
        u.ut_tv.tv_sec = libc::time(std::ptr::null_mut()) as _;
        libc::utmpxname(p.as_ptr());
        libc::setutxent();
        let r = libc::pututxline(&u);
        libc::endutxent();
        !r.is_null()
    }
}

pub fn current_username() -> Vec<u8> {
    unsafe {
        let pw = libc::getpwuid(libc::geteuid());
        if !pw.is_null() && !(*pw).pw_name.is_null() {
            let cs = std::ffi::CStr::from_ptr((*pw).pw_name);
            return cs.to_bytes().to_vec();
        }
    }
    b"root".to_vec()
}

pub fn fmt_time(out: &mut Vec<u8>, tv_sec: i64) {
    unsafe {
        let t: libc::time_t = tv_sec as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&t, &mut tm).is_null() {
            out.extend_from_slice(b"???");
            return;
        }
        let mut buf = [0 as libc::c_char; 64];
        let f = cstr(b"%a %b %e %H:%M").unwrap();
        let n = libc::strftime(buf.as_mut_ptr(), buf.len(), f.as_ptr(), &tm);
        if n == 0 {
            out.extend_from_slice(b"???");
            return;
        }
        let cs = std::ffi::CStr::from_ptr(buf.as_ptr());
        out.extend_from_slice(cs.to_bytes());
    }
}

pub fn init_run(argv0: &str, args: &[OsString]) -> Result<i32> {
    for a in args {
        if is_help(a) {
            return help_out(
                argv0,
                "[0123456SsQq]",
                "Change runlevel / init daemon (subset)",
            );
        }
    }
    let mut level: Option<u8> = None;
    let mut query = false;
    let mut i = 0;
    while i < args.len() {
        let b = args[i].as_bytes();
        if b == b"-q" || b == b"-Q" {
            query = true;
        } else if b.len() == 1 && (b"0123456SsQq".contains(&b[0])) {
            level = Some(b[0]);
        } else {
            eprintln!(
                "{}: invalid runlevel '{}'",
                argv0,
                String::from_utf8_lossy(b)
            );
            return Ok(1);
        }
        i += 1;
    }
    let pid1 = unsafe { libc::getpid() } == 1;
    if pid1 {
        return init_as_pid1(argv0, level, query);
    }

    if query {
        if unsafe { libc::kill(1, libc::SIGHUP) } != 0 {
            eprintln!(
                "{}: cannot signal init: {}",
                argv0,
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        return Ok(0);
    }
    match level {
        None => {
            eprintln!("{}: must be run as PID 1", argv0);
            Ok(1)
        }
        Some(lv) => {
            let prev = current_runlevel().unwrap_or(b'N');
            let pid = ((lv as i32) << 8) | (prev as i32);
            let utmp = std::env::var_os("BB_UTMP")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("/var/run/utmp"));
            if !write_utmpx(&utmp, UT_RUN_LVL, pid, b"~~", b"~~", b"runlevel") {
                eprintln!(
                    "{}: cannot write utmp: {}",
                    argv0,
                    io::Error::last_os_error()
                );
                return Ok(1);
            }

            if unsafe { libc::kill(1, libc::SIGHUP) } != 0 {
                eprintln!("{}: runlevel recorded, cannot signal init", argv0);
                return Ok(1);
            }
            Ok(0)
        }
    }
}

pub fn init_as_pid1(argv0: &str, _level: Option<u8>, _query: bool) -> Result<i32> {
    let data = fs::read("/etc/inittab").unwrap_or_default();
    let mut sysinit: Vec<Vec<u8>> = Vec::new();
    let mut respawn: Vec<Vec<u8>> = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        if line.is_empty() || line[0] == b'#' {
            continue;
        }
        let f: Vec<&[u8]> = line.split(|&b| b == b':').collect();
        if f.len() < 4 {
            continue;
        }
        match f[2] {
            b"sysinit" => sysinit.push(f[3].to_vec()),
            b"respawn" => respawn.push(f[3].to_vec()),
            _ => {}
        }
    }
    if sysinit.is_empty() && respawn.is_empty() {
        eprintln!("{}: no sysinit/respawn entries in /etc/inittab", argv0);
        return Ok(1);
    }
    for cmd in &sysinit {
        run_sh(cmd);
    }

    loop {
        let mut kids = Vec::new();
        for cmd in &respawn {
            let p = unsafe { libc::fork() };
            if p == 0 {
                let cs = match cstr(b"/bin/sh") {
                    Some(c) => c,
                    None => unsafe { libc::_exit(1) },
                };
                let a0 = cstr(b"sh").unwrap();
                let ac = cstr(b"-c").unwrap();
                let cc = match cstr(cmd) {
                    Some(c) => c,
                    None => unsafe { libc::_exit(1) },
                };
                let argv = [a0.as_ptr(), ac.as_ptr(), cc.as_ptr(), std::ptr::null()];
                unsafe {
                    libc::execv(cs.as_ptr(), argv.as_ptr());
                    libc::_exit(1);
                }
            } else if p > 0 {
                kids.push(p);
            }
        }
        if kids.is_empty() {
            eprintln!("{}: nothing to respawn", argv0);
            return Ok(1);
        }

        loop {
            let mut st = 0;
            let r = unsafe { libc::waitpid(-1, &mut st, 0) };
            if r < 0 {
                break;
            }
        }
    }
}

pub fn run_sh(cmd: &[u8]) {
    let p = unsafe { libc::fork() };
    if p == 0 {
        let cs = cstr(b"/bin/sh").unwrap();
        let a0 = cstr(b"sh").unwrap();
        let ac = cstr(b"-c").unwrap();
        let cc = cstr(cmd).unwrap_or_else(|| cstr(b"true").unwrap());
        let argv = [a0.as_ptr(), ac.as_ptr(), cc.as_ptr(), std::ptr::null()];
        unsafe {
            libc::execv(cs.as_ptr(), argv.as_ptr());
            libc::_exit(1);
        }
    } else if p > 0 {
        let mut st = 0;
        unsafe {
            libc::waitpid(p, &mut st, 0);
        }
    }
}

pub fn current_runlevel() -> Option<u8> {
    let utmp = std::env::var_os("BB_UTMP")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/var/run/utmp"));
    for e in read_utmpx(Some(&utmp)) {
        if e.typ == UT_RUN_LVL {
            let c = ((e.pid >> 8) & 0xff) as u8;
            if c != 0 {
                return Some(c);
            }
        }
    }
    None
}










pub fn reboot_run(
    name: &str,
    cmd: libc::c_int,
    allow_wtmp_only: bool,
    args: &[OsString],
) -> Result<i32> {
    for a in args {
        if is_help(a) {
            return help_out(
                name,
                "[-d SEC] [-n] [-f]",
                "Halt/reboot/power off the system (subset)",
            );
        }
    }
    let mut delay: u64 = 0;
    let mut force = false;
    let mut no_sync = false;
    let mut wtmp_only = false;
    let mut i = 0;
    while i < args.len() {
        let b = args[i].as_bytes();
        if b == b"-d" {
            i += 1;
            if i >= args.len() {
                eprintln!("{}: option requires an argument -- 'd'", name);
                return Ok(1);
            }
            delay = std::str::from_utf8(args[i].as_bytes())
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        } else if b.len() > 1 && b[0] == b'-' {
            for &c in &b[1..] {
                match c {
                    b'f' => force = true,
                    b'n' => no_sync = true,
                    b'w' => {
                        if !allow_wtmp_only {
                            eprintln!("{}: invalid option -- 'w'", name);
                            return Ok(1);
                        }
                        wtmp_only = true;
                    }
                    _ => {
                        eprintln!("{}: invalid option -- '{}'", name, c as char);
                        return Ok(1);
                    }
                }
            }
        } else {
            eprintln!(
                "{}: invalid argument '{}'",
                name,
                String::from_utf8_lossy(b)
            );
            return Ok(1);
        }
        i += 1;
    }
    if delay > 0 {
        let ts = libc::timespec {
            tv_sec: delay as libc::time_t,
            tv_nsec: 0,
        };

        unsafe {
            libc::nanosleep(&ts, std::ptr::null_mut());
        }
    }
    if !no_sync {
        unsafe { libc::sync() };
    }
    {
        let wtmp = std::env::var_os("BB_WTMP")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/var/log/wtmp"));
        if wtmp.exists() || std::env::var_os("BB_WTMP").is_some() {
            let user: &[u8] = if cmd == libc::LINUX_REBOOT_CMD_RESTART {
                b"reboot"
            } else {
                b"shutdown"
            };
            if !write_utmpx(&wtmp, UT_DEAD_PROCESS, 0, b"~~", b"~~", user) && wtmp.exists() {
                eprintln!(
                    "{}: cannot write wtmp: {}",
                    name,
                    io::Error::last_os_error()
                );
            }
        }
    }
    if wtmp_only {
        return Ok(0);
    }
    let pid1 = unsafe { libc::getpid() } == 1;
    if !force && !pid1 {
        eprintln!("{}: must be run as PID 1 (use -f to force)", name);
        return Ok(1);
    }

    if unsafe { libc::reboot(cmd) } != 0 {
        eprintln!("{}: cannot reboot: {}", name, io::Error::last_os_error());
        return Ok(1);
    }
    Ok(0)
}













pub fn bootchart_start(dir: &std::path::Path) -> Result<i32> {
    if let Err(e) = fs::create_dir_all(dir) {
        eprintln!("bootchartd: cannot create dir: {}", e);
        return Ok(1);
    }

    let p = unsafe { libc::fork() };
    if p < 0 {
        eprintln!("bootchartd: fork: {}", io::Error::last_os_error());
        return Ok(1);
    }
    if p > 0 {
        let pidfile = dir.join("bootchartd.pid");
        let _ = fs::write(&pidfile, format!("{}", p).as_bytes());
        return Ok(0);
    }
    unsafe {
        libc::setsid();
    }
    let log = dir.join("bootchart.log");
    let mut f = match OpenOptions::new().create(true).append(true).open(&log) {
        Ok(f) => f,
        Err(_) => unsafe { libc::_exit(1) },
    };
    loop {
        let now = unsafe { libc::time(std::ptr::null_mut()) };
        let stat = fs::read("/proc/stat").unwrap_or_default();
        let first = stat.split(|&b| b == b'\n').next().unwrap_or(b"");
        let load = fs::read("/proc/loadavg").unwrap_or_default();
        let load1 = load.split(|&b| b == b'\n').next().unwrap_or(b"");
        let mut line = Vec::with_capacity(256);
        line.extend_from_slice(format!("{}", now).as_bytes());
        line.push(b' ');
        line.extend_from_slice(first);
        line.push(b' ');
        line.extend_from_slice(load1);
        line.push(b'\n');
        if f.write_all(&line).is_err() {
            unsafe { libc::_exit(1) };
        }
        let _ = f.flush();
        let ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 200_000_000,
        };
        unsafe {
            libc::nanosleep(&ts, std::ptr::null_mut());
        }
    }
}

pub fn bootchart_stop(dir: &std::path::Path) -> Result<i32> {
    let pidfile = dir.join("bootchartd.pid");
    let data = match fs::read(&pidfile) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("bootchartd: not running (no pidfile)");
            return Ok(1);
        }
    };
    let pid: i32 = match std::str::from_utf8(&data)
        .ok()
        .and_then(|s| s.trim().parse().ok())
    {
        Some(p) => p,
        None => {
            eprintln!("bootchartd: bad pidfile");
            return Ok(1);
        }
    };

    if unsafe { libc::kill(pid, libc::SIGTERM) } != 0 {
        eprintln!(
            "bootchartd: cannot stop {}: {}",
            pid,
            io::Error::last_os_error()
        );
        return Ok(1);
    }
    let _ = fs::remove_file(&pidfile);
    Ok(0)
}

pub fn syslog_sock_path() -> std::path::PathBuf {
    std::env::var_os("BB_SYSLOG_SOCKET")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/dev/log"))
}

pub fn ring_path() -> std::path::PathBuf {
    std::env::var_os("BB_SYSLOG_RING")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/var/log/bb-ring"))
}

pub fn syslog_send(payload: &[u8]) {
    let sp = syslog_sock_path();
    let sc = match cstr(sp.as_os_str().as_bytes()) {
        Some(c) => c,
        None => return,
    };

    unsafe {
        let fd = libc::socket(libc::AF_UNIX, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC, 0);
        if fd < 0 {
            return;
        }
        let mut addr: libc::sockaddr_un = std::mem::zeroed();
        addr.sun_family = libc::AF_UNIX as libc::sa_family_t;
        let bytes = sc.as_bytes_with_nul();
        let n = bytes.len().min(addr.sun_path.len());
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr() as *const libc::c_char,
            addr.sun_path.as_mut_ptr(),
            n,
        );
        let alen = (std::mem::size_of::<libc::sa_family_t>() + n) as libc::socklen_t;
        libc::sendto(
            fd,
            payload.as_ptr() as *const libc::c_void,
            payload.len(),
            libc::MSG_NOSIGNAL,
            &addr as *const _ as *const libc::sockaddr,
            alen,
        );
        libc::close(fd);
    }
}




pub fn klog_forward() -> Result<i32> {
    let src = std::env::var_os("BB_KLOG_FILE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/proc/kmsg"));
    let mut f = match fs::File::open(&src) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("klogd: cannot open {}: {}", src.display(), e);
            return Ok(1);
        }
    };
    let mut tail: Vec<u8> = Vec::new();
    let mut buf = [0u8; 8192];
    let from_file = std::env::var_os("BB_KLOG_FILE").is_some();
    loop {
        match f.read(&mut buf) {
            Ok(0) => {
                if from_file {
                    break;
                }

                let ts = libc::timespec {
                    tv_sec: 1,
                    tv_nsec: 0,
                };
                unsafe {
                    libc::nanosleep(&ts, std::ptr::null_mut());
                }
                continue;
            }
            Ok(n) => {
                let mut start = 0;
                for (k, &c) in buf[..n].iter().enumerate() {
                    if c == b'\n' {
                        tail.extend_from_slice(&buf[start..k]);
                        klog_emit(&tail);
                        tail.clear();
                        start = k + 1;
                    }
                }
                tail.extend_from_slice(&buf[start..n]);
            }
            Err(e) => {
                eprintln!("klogd: read error: {}", e);
                return Ok(1);
            }
        }
    }
    if !tail.is_empty() {
        klog_emit(&tail);
    }
    Ok(0)
}

pub fn klog_emit(rec: &[u8]) {
    if rec.is_empty() {
        return;
    }

    let mut payload: Vec<u8> = Vec::with_capacity(rec.len() + 4);
    if let Some(semi) = rec.iter().position(|&b| b == b';') {
        let pri: Vec<u8> = rec[..semi]
            .iter()
            .take_while(|&&b| b != b',')
            .copied()
            .collect();
        payload.push(b'<');
        payload.extend_from_slice(&pri);
        payload.push(b'>');
        payload.extend_from_slice(&rec[semi + 1..]);
    } else {
        payload.extend_from_slice(rec);
    }
    payload.push(b'\n');
    syslog_send(&payload);
}




pub fn syslog_loop(outfile: &std::path::Path, ring_cap: usize, mark_min: u64) -> Result<i32> {
    let sp = syslog_sock_path();
    let sc = match cstr(sp.as_os_str().as_bytes()) {
        Some(c) => c,
        None => {
            eprintln!("syslogd: bad socket path");
            return Ok(1);
        }
    };

    let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC, 0) };
    if fd < 0 {
        eprintln!("syslogd: socket: {}", io::Error::last_os_error());
        return Ok(1);
    }
    let _sock = unsafe { fs::File::from_raw_fd(fd) };
    unsafe {
        let mut addr: libc::sockaddr_un = std::mem::zeroed();
        addr.sun_family = libc::AF_UNIX as libc::sa_family_t;
        let bytes = sc.as_bytes_with_nul();
        let n = bytes.len().min(addr.sun_path.len());
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr() as *const libc::c_char,
            addr.sun_path.as_mut_ptr(),
            n,
        );
        let alen = (std::mem::size_of::<libc::sa_family_t>() + n) as libc::socklen_t;

        libc::unlink(sc.as_ptr());
        if libc::bind(fd, &addr as *const _ as *const libc::sockaddr, alen) != 0 {
            eprintln!(
                "syslogd: bind {}: {}",
                sp.display(),
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        if libc::chmod(sc.as_ptr(), 0o666) != 0 {
            eprintln!("syslogd: chmod: {}", io::Error::last_os_error());
            return Ok(1);
        }
    }
    let mut logf = match OpenOptions::new().create(true).append(true).open(outfile) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("syslogd: cannot open {}: {}", outfile.display(), e);
            return Ok(1);
        }
    };
    let mut ring: Vec<u8> = Vec::new();
    let mut buf = [0u8; 65536];
    let mut last_mark = unsafe { libc::time(std::ptr::null_mut()) };
    loop {
        let mut rfds: libc::fd_set = unsafe { std::mem::zeroed() };
        unsafe {
            libc::FD_ZERO(&mut rfds);
            libc::FD_SET(fd, &mut rfds);
        }
        let mut tv = libc::timeval {
            tv_sec: 5,
            tv_usec: 0,
        };

        let r = unsafe {
            libc::select(
                fd + 1,
                &mut rfds,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut tv,
            )
        };
        let now = unsafe { libc::time(std::ptr::null_mut()) };
        if mark_min > 0 && (now - last_mark) >= (mark_min as i64 * 60) {
            last_mark = now;
            let mark = b"-- MARK --\n";
            let _ = logf.write_all(mark);
            ring_push(&mut ring, mark, ring_cap);
            persist_ring(&ring);
        }
        if r <= 0 {
            continue;
        }

        let n = unsafe {
            libc::recvfrom(
                fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if n <= 0 {
            continue;
        }
        let mut msg = &buf[..n as usize];
        while msg.last() == Some(&b'\n') || msg.last() == Some(&0) {
            msg = &msg[..msg.len() - 1];
        }
        let _ = logf.write_all(msg);
        let _ = logf.write_all(b"\n");
        let mut line = msg.to_vec();
        line.push(b'\n');
        ring_push(&mut ring, &line, ring_cap);
        persist_ring(&ring);
    }
}

pub fn ring_push(ring: &mut Vec<u8>, line: &[u8], cap: usize) {
    ring.extend_from_slice(line);
    if ring.len() > cap {
        let drop = ring.len() - cap;

        let mut cut = drop;
        if let Some(p) = ring[drop..].iter().position(|&b| b == b'\n') {
            cut = drop + p + 1;
        }
        ring.drain(..cut.min(ring.len()));
    }
}

pub fn persist_ring(ring: &[u8]) {
    let rp = ring_path();
    if let Some(parent) = rp.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }
    let tmp = rp.with_extension("tmp");
    if fs::write(&tmp, ring).is_ok() {
        let _ = fs::rename(&tmp, &rp);
    }
}




#[derive(Clone)]
pub struct CronEntry {
    pub min: Vec<u8>,
    pub hr: Vec<u8>,
    pub dom: Vec<u8>,
    pub mon: Vec<u8>,
    pub dow: Vec<u8>,
    pub cmd: Vec<u8>,
    pub dom_star: bool,
    pub dow_star: bool,
}

pub fn cron_dir() -> std::path::PathBuf {
    std::env::var_os("BB_CRON_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("CRON_DIR").map(std::path::PathBuf::from))
        .unwrap_or_else(|| std::path::PathBuf::from("/var/spool/cron/crontabs"))
}

pub fn parse_cron_field(f: &[u8], lo: i64, hi: i64, names: Option<&[(&[u8], i64)]>) -> Option<Vec<u8>> {
    let size = (hi - lo + 1) as usize;
    let mut bits = vec![0u8; size];
    if f == b"*" {
        for b in bits.iter_mut() {
            *b = 1;
        }
        return Some(bits);
    }
    for part in f.split(|&b| b == b',') {
        if part.is_empty() {
            return None;
        }
        let (range, step) = match part.iter().position(|&b| b == b'/') {
            Some(p) => (&part[..p], Some(&part[p + 1..])),
            None => (part, None),
        };
        let st: i64 = match step {
            Some(s) => std::str::from_utf8(s).ok()?.parse().ok()?,
            None => 1,
        };
        if st <= 0 {
            return None;
        }
        let (mut a, mut b2) = if range == b"*" {
            (lo, hi)
        } else if let Some(p) = range.iter().position(|&b| b == b'-') {
            (
                cron_num(&range[..p], names)?,
                cron_num(&range[p + 1..], names)?,
            )
        } else {
            let v = cron_num(range, names)?;
            (v, v)
        };
        if a < lo {
            a = lo;
        }
        if b2 > hi {
            b2 = hi;
        }
        if a > b2 {
            return None;
        }
        let mut v = a;

        while v <= b2 {
            bits[(v - lo) as usize] = 1;
            v += st;
        }
    }
    Some(bits)
}

pub fn cron_num(s: &[u8], names: Option<&[(&[u8], i64)]>) -> Option<i64> {
    if let Some(ns) = names {
        let mut lower = [0u8; 16];
        let n = s.len().min(16);
        for (i, &b) in s[..n].iter().enumerate() {
            lower[i] = b.to_ascii_lowercase();
        }
        for (nm, v) in ns {
            if *nm == &lower[..n] {
                return Some(*v);
            }
        }
    }
    std::str::from_utf8(s).ok()?.parse::<i64>().ok()
}

pub const MON_NAMES: [(&[u8], i64); 12] = [
    (b"jan", 1),
    (b"feb", 2),
    (b"mar", 3),
    (b"apr", 4),
    (b"may", 5),
    (b"jun", 6),
    (b"jul", 7),
    (b"aug", 8),
    (b"sep", 9),
    (b"oct", 10),
    (b"nov", 11),
    (b"dec", 12),
];
pub const DOW_NAMES: [(&[u8], i64); 8] = [
    (b"sun", 0),
    (b"mon", 1),
    (b"tue", 2),
    (b"wed", 3),
    (b"thu", 4),
    (b"fri", 5),
    (b"sat", 6),
    (b"sun", 0),
];

pub fn parse_crontab(data: &[u8]) -> (Vec<CronEntry>, Vec<Vec<u8>>) {
    let mut out = Vec::new();
    let mut errs = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        let mut cut = line.len();
        for (idx, w) in line.windows(1).enumerate() {
            if w[0] == b'#' && (idx == 0 || line[idx - 1] == b' ' || line[idx - 1] == b'\t') {
                cut = idx;
                break;
            }
        }
        let t = trim(&line[..cut]);
        if t.is_empty() {
            continue;
        }
        let mut it = t
            .split(|&b| b == b' ' || b == b'\t')
            .filter(|s| !s.is_empty());
        let f: Vec<&[u8]> = it.by_ref().take(6).collect();
        if f.len() < 6 {
            if t.contains(&b'=') {
                continue;
            }
            errs.push(t.to_vec());
            continue;
        }

        let mut pos = 0;
        let mut fields: Vec<&[u8]> = Vec::new();
        {
            let mut k = 0;
            while k < t.len() && fields.len() < 5 {
                while k < t.len() && (t[k] == b' ' || t[k] == b'\t') {
                    k += 1;
                }
                let s = k;
                while k < t.len() && t[k] != b' ' && t[k] != b'\t' {
                    k += 1;
                }
                if s < k {
                    fields.push(&t[s..k]);
                }
                pos = k;
            }
        }
        if fields.len() < 5 {
            errs.push(t.to_vec());
            continue;
        }
        while pos < t.len() && (t[pos] == b' ' || t[pos] == b'\t') {
            pos += 1;
        }
        let cmd = &t[pos..];
        if cmd.is_empty() {
            errs.push(t.to_vec());
            continue;
        }
        let mon = parse_cron_field(fields[3], 1, 12, Some(&MON_NAMES));
        let dow_raw = fields[4];

        let dow_norm: Vec<u8> = dow_raw
            .split(|&b| b == b',')
            .map(|p| if p == b"7" { b"0".to_vec() } else { p.to_vec() })
            .collect::<Vec<_>>()
            .join(&b',');
        let dow = parse_cron_field(&dow_norm, 0, 6, Some(&DOW_NAMES));
        let entry = match (
            parse_cron_field(fields[0], 0, 59, None),
            parse_cron_field(fields[1], 0, 23, None),
            parse_cron_field(fields[2], 1, 31, None),
            mon,
            dow,
        ) {
            (Some(a), Some(b), Some(c), Some(d), Some(e)) => CronEntry {
                min: a,
                hr: b,
                dom: c,
                mon: d,
                dow: e,
                cmd: cmd.to_vec(),
                dom_star: fields[2] == b"*",
                dow_star: fields[4] == b"*",
            },
            _ => {
                errs.push(t.to_vec());
                continue;
            }
        };
        out.push(entry);
    }
    (out, errs)
}

pub fn trim(b: &[u8]) -> &[u8] {
    let mut s = 0;
    let mut e = b.len();
    while s < e && (b[s] == b' ' || b[s] == b'\t' || b[s] == b'\r') {
        s += 1;
    }
    while e > s && (b[e - 1] == b' ' || b[e - 1] == b'\t' || b[e - 1] == b'\r') {
        e -= 1;
    }
    &b[s..e]
}

pub fn cron_match(e: &CronEntry, min: i64, hr: i64, dom: i64, mon: i64, dow: i64) -> bool {
    if e.min.get(min as usize).copied().unwrap_or(0) == 0 {
        return false;
    }
    if e.hr.get(hr as usize).copied().unwrap_or(0) == 0 {
        return false;
    }
    if e.mon.get((mon - 1) as usize).copied().unwrap_or(0) == 0 {
        return false;
    }
    let dom_ok = e.dom.get((dom - 1) as usize).copied().unwrap_or(0) != 0;
    let dow_ok = e.dow.get(dow as usize).copied().unwrap_or(0) != 0;

    if e.dom_star && e.dow_star {
        true
    } else if e.dom_star {
        dow_ok
    } else if e.dow_star {
        dom_ok
    } else {
        dom_ok || dow_ok
    }
}

pub fn spawn_job(cmd: &[u8]) {
    let p = unsafe { libc::fork() };
    if p != 0 {
        return;
    }
    let cs = cstr(b"/bin/sh").unwrap();
    let a0 = cstr(b"sh").unwrap();
    let ac = cstr(b"-c").unwrap();
    match cstr(cmd) {
        Some(cc) => {
            let argv = [a0.as_ptr(), ac.as_ptr(), cc.as_ptr(), std::ptr::null()];
            unsafe {
                libc::execv(cs.as_ptr(), argv.as_ptr());
                libc::_exit(127);
            }
        }
        None => unsafe { libc::_exit(127) },
    }
}




pub struct CronLog {
    pub file: Option<fs::File>,
    pub use_syslog: bool,
}

impl CronLog {
    fn log(&mut self, level: u32, thresh: u32, msg: &str) {
        if level > thresh {
            return;
        }
        eprintln!("crond: {}", msg);
        if let Some(f) = self.file.as_mut() {
            let _ = writeln!(f, "crond: {}", msg);
        }
        if self.use_syslog {
            let mut p = Vec::with_capacity(msg.len() + 16);
            p.extend_from_slice(b"<13>crond: ");
            p.extend_from_slice(msg.as_bytes());
            p.push(b'\n');
            syslog_send(&p);
        }
    }
}

pub fn crond_loop(
    dir: &std::path::Path,
    debug: u32,
    log_level: u32,
    log_syslog: bool,
    log_file: Option<&std::path::Path>,
) -> Result<i32> {
    let mut clog = CronLog {
        file: log_file.and_then(|p| OpenOptions::new().create(true).append(true).open(p).ok()),
        use_syslog: log_syslog,
    };

    let mut last_fired: i64 = -1;
    loop {
        let now = unsafe { libc::time(std::ptr::null_mut()) };
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        unsafe {
            libc::localtime_r(&now, &mut tm);
        }
        let minute_id = now / 60;
        if minute_id != last_fired {
            last_fired = minute_id;
            let dow = ((tm.tm_wday % 7 + 7) % 7) as i64;
            if debug > 0 {
                clog.log(
                    5,
                    log_level,
                    &format!("scan {:02}:{:02} {}", tm.tm_hour, tm.tm_min, dir.display()),
                );
            }
            let rd = fs::read_dir(dir);
            if let Ok(rd) = rd {
                for ent in rd.flatten() {
                    let data = fs::read(ent.path()).unwrap_or_default();
                    let (entries, errs) = parse_crontab(&data);
                    for e in &errs {
                        clog.log(
                            8,
                            log_level,
                            &format!(
                                "{}: ignoring bad line '{}'",
                                ent.path().display(),
                                String::from_utf8_lossy(e)
                            ),
                        );
                    }
                    for e in &entries {
                        if cron_match(
                            e,
                            tm.tm_min as i64,
                            tm.tm_hour as i64,
                            tm.tm_mday as i64,
                            (tm.tm_mon + 1) as i64,
                            dow,
                        ) {
                            if debug > 0 {
                                clog.log(
                                    5,
                                    log_level,
                                    &format!("run '{}'", String::from_utf8_lossy(&e.cmd)),
                                );
                            }
                            spawn_job(&e.cmd);
                        }
                    }
                }
            }
        }

        loop {
            let mut st = 0;

            let r = unsafe { libc::waitpid(-1, &mut st, libc::WNOHANG) };
            if r <= 0 {
                break;
            }
        }

        let now2 = unsafe { libc::time(std::ptr::null_mut()) };
        let sleep_s = (60 - (now2 % 60) + 2) % 60 + 1;
        let ts = libc::timespec {
            tv_sec: sleep_s,
            tv_nsec: 0,
        };
        unsafe {
            libc::nanosleep(&ts, std::ptr::null_mut());
        }
    }
}




pub fn crontab_edit(tname: &str, path: &std::path::Path) -> Result<i32> {
    let old = fs::read(path).unwrap_or_default();
    let mut tmpp = std::env::temp_dir();
    tmpp.push(format!("crontab-{}.{}", tname, unsafe { libc::getpid() }));
    if fs::write(&tmpp, &old).is_err() {
        eprintln!("crontab: cannot write temp file");
        return Ok(1);
    }
    let editor = std::env::var_os("VISUAL")
        .or_else(|| std::env::var_os("EDITOR"))
        .unwrap_or_else(|| OsString::from("vi"));

    let p = unsafe { libc::fork() };
    if p < 0 {
        eprintln!("crontab: fork: {}", io::Error::last_os_error());
        return Ok(1);
    }
    if p == 0 {
        let prog = cstr(editor.as_bytes()).unwrap_or_else(|| cstr(b"vi").unwrap());
        let a0 = prog.clone();
        let fc = match cstr(tmpp.as_os_str().as_bytes()) {
            Some(c) => c,
            None => unsafe { libc::_exit(1) },
        };
        let argv = [a0.as_ptr(), fc.as_ptr(), std::ptr::null()];
        unsafe {
            libc::execvp(prog.as_ptr(), argv.as_ptr());
            libc::_exit(127);
        }
    }
    let mut st = 0;
    unsafe {
        libc::waitpid(p, &mut st, 0);
    }
    let new = fs::read(&tmpp).unwrap_or_default();
    let _ = fs::remove_file(&tmpp);
    if new == old {
        eprintln!("crontab: no changes made");
        return Ok(0);
    }
    let (_, errs) = parse_crontab(&new);
    for e in &errs {
        eprintln!(
            "crontab: warning: ignoring bad line '{}'",
            String::from_utf8_lossy(e)
        );
    }
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match fs::write(path, &new) {
        Ok(()) => Ok(0),
        Err(e) => {
            eprintln!("crontab: cannot install: {}", e);
            Ok(1)
        }
    }
}

pub fn baud_const(n: u32) -> Option<libc::speed_t> {
    Some(match n {
        0 => 0,
        50 => libc::B50,
        75 => libc::B75,
        110 => libc::B110,
        134 => libc::B134,
        150 => libc::B150,
        200 => libc::B200,
        300 => libc::B300,
        600 => libc::B600,
        1200 => libc::B1200,
        1800 => libc::B1800,
        2400 => libc::B2400,
        4800 => libc::B4800,
        9600 => libc::B9600,
        19200 => libc::B19200,
        38400 => libc::B38400,
        57600 => libc::B57600,
        115200 => libc::B115200,
        230400 => libc::B230400,
        460800 => libc::B460800,
        921600 => libc::B921600,
        _ => return None,
    })
}




pub fn getty_run(
    tty: &std::path::Path,
    baud: libc::speed_t,
    issue: Option<&std::path::Path>,
    login_prog: &std::path::Path,
    no_issue: bool,
    no_prompt: bool,
    timeout: u64,
) -> Result<i32> {
    let tc = match cstr(tty.as_os_str().as_bytes()) {
        Some(c) => c,
        None => {
            eprintln!("getty: bad tty path");
            return Ok(1);
        }
    };

    let fd = unsafe { libc::open(tc.as_ptr(), libc::O_RDWR | libc::O_NOCTTY) };
    if fd < 0 {
        eprintln!(
            "getty: cannot open {}: {}",
            tty.display(),
            io::Error::last_os_error()
        );
        return Ok(1);
    }

    unsafe {
        libc::setsid();
        libc::ioctl(fd, libc::TIOCSCTTY as _, 0);
        libc::dup2(fd, 0);
        libc::dup2(fd, 1);
        libc::dup2(fd, 2);
        if fd > 2 {
            libc::close(fd);
        }
    }

    unsafe {
        let mut t: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(0, &mut t) == 0 {
            libc::cfsetispeed(&mut t, baud);
            libc::cfsetospeed(&mut t, baud);
            t.c_cflag |= libc::CREAD | libc::CS8;
            t.c_lflag |= libc::ISIG | libc::ICANON | libc::ECHO | libc::ECHOE | libc::ECHOK;
            t.c_iflag |= libc::BRKINT | libc::ICRNL | libc::IXON | libc::ISTRIP;
            t.c_oflag |= libc::OPOST | libc::ONLCR;
            libc::tcsetattr(0, libc::TCSANOW, &t);
        }
    }
    let stdout = io::stdout();
    let mut o = stdout.lock();
    if !no_issue {
        let ipath = issue
            .map(|p| p.to_path_buf())
            .or_else(|| std::env::var_os("BB_ISSUE_FILE").map(std::path::PathBuf::from))
            .unwrap_or_else(|| std::path::PathBuf::from("/etc/issue"));
        if let Ok(d) = fs::read(&ipath) {
            let _ = o.write_all(&d);
            if !d.ends_with(b"\n") {
                let _ = o.write_all(b"\n");
            }
        }
    }
    if !no_prompt {
        let _ = o.write_all(tty.as_os_str().as_bytes());
        let _ = o.write_all(b" login: ");
        let _ = o.flush();
    } else {
        let _ = o.flush();
    }
    drop(o);

    let mut name: Vec<u8> = Vec::new();
    if timeout > 0 {
        let mut rfds: libc::fd_set = unsafe { std::mem::zeroed() };
        unsafe {
            libc::FD_ZERO(&mut rfds);
            libc::FD_SET(0, &mut rfds);
        }
        let mut tv = libc::timeval {
            tv_sec: timeout as libc::time_t,
            tv_usec: 0,
        };

        let r = unsafe {
            libc::select(
                1,
                &mut rfds,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut tv,
            )
        };
        if r <= 0 {
            eprintln!("getty: timed out");
            return Ok(1);
        }
    }
    let mut buf = [0u8; 256];
    loop {
        let n = io::stdin().lock().read(&mut buf).unwrap_or(0);
        if n == 0 {
            break;
        }
        name.extend_from_slice(&buf[..n]);
        if name.contains(&b'\n') {
            break;
        }
        if name.len() > 64 {
            break;
        }
    }
    let name = trim(&name);
    let name = match name.iter().position(|&b| b == b'\n') {
        Some(p) => &name[..p],
        None => name,
    };
    let name = trim(name);
    if name.is_empty() || name.contains(&b' ') {
        eprintln!("getty: bad login name");
        return Ok(1);
    }
    let prog = match cstr(login_prog.as_os_str().as_bytes()) {
        Some(c) => c,
        None => {
            eprintln!("getty: bad login path");
            return Ok(1);
        }
    };
    let uname = match cstr(name) {
        Some(c) => c,
        None => {
            eprintln!("getty: bad login name");
            return Ok(1);
        }
    };

    unsafe {
        libc::execv(prog.as_ptr(), [uname.as_ptr(), std::ptr::null()].as_ptr());
        eprintln!(
            "getty: cannot exec {}: {}",
            login_prog.display(),
            io::Error::last_os_error()
        );
        libc::_exit(1);
    }
}









