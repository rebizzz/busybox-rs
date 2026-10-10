use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

fn push_u64(out: &mut Vec<u8>, mut v: u64) {
    if v == 0 {
        out.push(b'0');
        return;
    }
    let mut tmp = [0u8; 20];
    let mut n = 0;
    while v > 0 {
        tmp[n] = b'0' + (v % 10) as u8;
        v /= 10;
        n += 1;
    }
    while n > 0 {
        n -= 1;
        out.push(tmp[n]);
    }
}

fn read_small(path: &str, buf: &mut [u8]) -> usize {
    let mut f = match File::open(path) {
        Ok(f) => f,
        Err(_) => return 0,
    };
    let mut n = 0;
    while n < buf.len() {
        match f.read(&mut buf[n..]) {
            Ok(0) => break,
            Ok(r) => n += r,
            Err(_) => break,
        }
    }
    n
}

fn proc_path(pid: u32, tail: &[u8], out: &mut [u8; 48]) -> usize {
    out[..6].copy_from_slice(b"/proc/");
    let mut len = 6;
    let mut tmp = [0u8; 12];
    let mut tl = 0;
    let mut v = pid;
    if v == 0 {
        tmp[0] = b'0';
        tl = 1;
    } else {
        let mut rev = [0u8; 12];
        let mut rn = 0;
        while v > 0 {
            rev[rn] = b'0' + (v % 10) as u8;
            v /= 10;
            rn += 1;
        }
        while rn > 0 {
            rn -= 1;
            tmp[tl] = rev[rn];
            tl += 1;
        }
    }
    for &c in &tmp[..tl] {
        if len < out.len() {
            out[len] = c;
            len += 1;
        }
    }
    for &c in tail {
        if len < out.len() {
            out[len] = c;
            len += 1;
        }
    }
    len
}

fn proc_pids() -> Vec<u32> {
    let mut v = Vec::new();
    let dir = match std::fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return v,
    };
    for e in dir.flatten() {
        let b = e.file_name();
        let bb = b.as_bytes();
        if bb.is_empty() || !bb.iter().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let mut pid: u32 = 0;
        for &c in bb {
            pid = pid.saturating_mul(10).saturating_add((c - b'0') as u32);
        }
        v.push(pid);
    }
    v
}

struct PStat {
    pid: u32,
    comm: Vec<u8>,
    state: u8,
    utime: u64,
    stime: u64,
    starttime: u64,
    vsize: u64,
    rss_pages: i64,
}

fn parse_num(b: &[u8]) -> u64 {
    let mut v: u64 = 0;
    for &c in b {
        if c.is_ascii_digit() {
            v = v.saturating_mul(10).saturating_add((c - b'0') as u64);
        } else if c == b' ' || c == b'\t' {
            if v > 0 || b.len() == 1 {
                break;
            }
        } else {
            break;
        }
    }
    v
}

fn read_stat(pid: u32) -> Option<PStat> {
    let mut path = [0u8; 48];
    let n = proc_path(pid, b"/stat", &mut path);
    let ps = std::str::from_utf8(&path[..n]).ok()?;
    let mut buf = [0u8; 1024];
    let r = read_small(ps, &mut buf);
    if r == 0 {
        return None;
    }
    let b = &buf[..r];
    let open = b.iter().position(|&c| c == b'(')?;
    let close = b[open..].iter().position(|&c| c == b')')? + open;
    let pidv: u32 = std::str::from_utf8(&b[..open]).ok()?.trim().parse().ok()?;
    let comm = b[open + 1..close].to_vec();
    let rest = &b[close + 2..];

    let mut f = rest.split(|&c| c == b' ');
    let state = *f.next()?.first()?;
    f.next()?;
    for _ in 0..9 {
        f.next()?;
    }
    let utime = parse_num(f.next()?);
    let stime = parse_num(f.next()?);
    for _ in 0..6 {
        f.next()?;
    }
    let starttime = parse_num(f.next()?);
    let vsize = parse_num(f.next()?);
    let rss_pages: i64 = std::str::from_utf8(f.next()?)
        .ok()?
        .trim()
        .parse()
        .unwrap_or(0);
    Some(PStat {
        pid: pidv,
        comm,
        state,
        utime,
        stime,
        starttime,
        vsize,
        rss_pages,
    })
}

fn read_cmdline(pid: u32, buf: &mut [u8]) -> usize {
    let mut path = [0u8; 48];
    let n = proc_path(pid, b"/cmdline", &mut path);
    match std::str::from_utf8(&path[..n]) {
        Ok(ps) => read_small(ps, buf),
        Err(_) => 0,
    }
}

fn read_uids(pid: u32) -> Option<(u32, u32)> {
    let mut path = [0u8; 48];
    let n = proc_path(pid, b"/status", &mut path);
    let ps = std::str::from_utf8(&path[..n]).ok()?;
    let mut buf = [0u8; 2048];
    let r = read_small(ps, &mut buf);
    let b = &buf[..r];
    let mut i = 0;
    while i < b.len() {
        if b[i..].starts_with(b"Uid:") {
            let mut nums = [0u32; 2];
            let mut k = 0;
            let mut j = i + 4;
            while j < b.len() && k < 2 {
                while j < b.len() && (b[j] == b' ' || b[j] == b'\t') {
                    j += 1;
                }
                if j < b.len() && b[j].is_ascii_digit() {
                    let mut v: u32 = 0;
                    while j < b.len() && b[j].is_ascii_digit() {
                        v = v.saturating_mul(10).saturating_add((b[j] - b'0') as u32);
                        j += 1;
                    }
                    nums[k] = v;
                    k += 1;
                } else {
                    break;
                }
            }
            if k == 2 {
                return Some((nums[0], nums[1]));
            }
            return None;
        }
        while i < b.len() && b[i] != b'\n' {
            i += 1;
        }
        i += 1;
    }
    None
}

fn uid_of_name(name: &[u8]) -> Option<u32> {
    if !name.is_empty() && name.iter().all(|c| c.is_ascii_digit()) {
        let mut v: u32 = 0;
        for &c in name {
            v = v.saturating_mul(10).saturating_add((c - b'0') as u32);
        }
        return Some(v);
    }
    let mut nb = [0u8; 65];
    if name.len() >= nb.len() {
        return None;
    }
    nb[..name.len()].copy_from_slice(name);
    unsafe {
        let pw = libc::getpwnam(nb.as_ptr() as *const libc::c_char);
        if pw.is_null() {
            return None;
        }
        Some((*pw).pw_uid)
    }
}

fn user_name(uid: u32) -> Vec<u8> {
    unsafe {
        let pw = libc::getpwuid(uid);
        if !pw.is_null() && !(*pw).pw_name.is_null() {
            let p = (*pw).pw_name as *const u8;
            let mut n = 0;
            while *p.add(n) != 0 {
                n += 1;
            }
            return std::slice::from_raw_parts(p, n).to_vec();
        }
    }
    uid.to_string().into_bytes()
}

fn strerror_last() -> String {
    let e = std::io::Error::last_os_error();
    match e.raw_os_error() {
        Some(n) => unsafe {
            let p = libc::strerror(n);
            if p.is_null() {
                e.to_string()
            } else {
                let mut len = 0;
                while *p.add(len) != 0 {
                    len += 1;
                }
                String::from_utf8_lossy(std::slice::from_raw_parts(p as *const u8, len))
                    .into_owned()
            }
        },
        None => e.to_string(),
    }
}

fn sig_from_name(s: &[u8]) -> Option<i32> {
    const TABLE: &[(&[u8], i32)] = &[
        (b"HUP", 1),
        (b"INT", 2),
        (b"QUIT", 3),
        (b"ILL", 4),
        (b"ABRT", 6),
        (b"FPE", 8),
        (b"KILL", 9),
        (b"USR1", 10),
        (b"SEGV", 11),
        (b"USR2", 12),
        (b"PIPE", 13),
        (b"ALRM", 14),
        (b"TERM", 15),
        (b"CHLD", 17),
        (b"CONT", 18),
        (b"STOP", 19),
        (b"TSTP", 20),
        (b"URG", 23),
    ];
    let mut name = s;
    if name.len() > 3 && name[..3].eq_ignore_ascii_case(b"SIG") {
        name = &name[3..];
    }
    if !name.is_empty() && name.iter().all(|c| c.is_ascii_digit()) {
        let mut v: i32 = 0;
        for &c in name {
            v = v.saturating_mul(10).saturating_add((c - b'0') as i32);
        }
        return Some(v);
    }
    TABLE
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| *v)
}

fn exec_prog(prog: &[u8], args: &[OsString]) -> i32 {
    exec_prog_argv0(prog, prog, args)
}

fn exec_prog_argv0(prog: &[u8], argv0: &[u8], args: &[OsString]) -> i32 {
    use std::ffi::CString;
    let c_bin = CString::new(prog).unwrap_or_else(|_| CString::new("sh").unwrap());
    let c_argv0 = CString::new(argv0).unwrap_or_else(|_| CString::new("sh").unwrap());
    let mut cs: Vec<CString> = Vec::with_capacity(args.len() + 1);
    cs.push(c_argv0);
    for a in args {
        cs.push(CString::new(a.as_bytes()).unwrap_or_else(|_| CString::new("").unwrap()));
    }
    let mut ptrs: Vec<*const libc::c_char> = cs.iter().map(|c| c.as_ptr()).collect();
    ptrs.push(std::ptr::null());
    unsafe {
        libc::execvp(c_bin.as_ptr(), ptrs.as_ptr());
    }
    eprintln!(
        "{}: {}",
        String::from_utf8_lossy(prog),
        std::io::Error::last_os_error()
    );
    127
}

struct PgrepOpts<'a> {
    full: bool,
    exact: bool,
    newest: bool,
    oldest: bool,
    invert: bool,
    count: bool,
    list: bool,
    uid: Option<u32>,
    euid: Option<u32>,
    signal: i32,
    pattern: Option<&'a [u8]>,
}

fn parse_pgrep(
    args: &[OsString],
    is_pkill: bool,
) -> std::result::Result<(PgrepOpts<'_>, Vec<&[u8]>), i32> {
    let mut o = PgrepOpts {
        full: false,
        exact: false,
        newest: false,
        oldest: false,
        invert: false,
        count: false,
        list: false,
        uid: None,
        euid: None,
        signal: libc::SIGTERM,
        pattern: None,
    };
    let mut pats: Vec<&[u8]> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = args[i].as_bytes();
        if b == b"--" {
            i += 1;
            while i < args.len() {
                pats.push(args[i].as_bytes());
                i += 1;
            }
            break;
        } else if b.len() > 1 && b[0] == b'-' && !b[1].is_ascii_digit() {
            let mut j = 1;
            while j < b.len() {
                match b[j] {
                    b'f' => o.full = true,
                    b'x' => o.exact = true,
                    b'n' => o.newest = true,
                    b'o' => o.oldest = true,
                    b'v' => o.invert = true,
                    b'c' => o.count = true,
                    b'l' => o.list = true,
                    b'u' | b'U' => {
                        let is_e = b[j] == b'U';
                        let val: &[u8] = if j + 1 < b.len() {
                            &b[j + 1..]
                        } else {
                            i += 1;
                            if i >= args.len() {
                                eprintln!("pgrep: option requires an argument");
                                return Err(1);
                            }
                            args[i].as_bytes()
                        };

                        let mut got = None;
                        for part in val.split(|&c| c == b',') {
                            if let Some(u) = uid_of_name(part) {
                                got = Some(u);
                                break;
                            }
                        }
                        match got {
                            Some(u) => {
                                if is_e {
                                    o.euid = Some(u);
                                } else {
                                    o.uid = Some(u);
                                }
                            }
                            None => {
                                eprintln!("pgrep: invalid user");
                                return Err(1);
                            }
                        }
                        break;
                    }
                    b's' => {
                        i += 1;
                        if i >= args.len() {
                            eprintln!("pgrep: -s needs a signal");
                            return Err(1);
                        }
                        match sig_from_name(args[i].as_bytes()) {
                            Some(s) => o.signal = s,
                            None => {
                                eprintln!("pgrep: unknown signal");
                                return Err(1);
                            }
                        }
                        break;
                    }
                    _ => {
                        eprintln!("pgrep: invalid option -- '{}'", b[j] as char);
                        return Err(1);
                    }
                }
                j += 1;
            }
        } else if is_pkill && b.len() > 1 && b[0] == b'-' && b[1].is_ascii_digit() {
            match sig_from_name(&b[1..]) {
                Some(s) => o.signal = s,
                None => {
                    eprintln!("pkill: unknown signal");
                    return Err(1);
                }
            }
        } else {
            pats.push(b);
        }
        i += 1;
    }
    if pats.is_empty() {
        eprintln!(
            "usage: {} [-fxnovcl] [-u|-U user] {}pattern",
            if is_pkill { "pkill" } else { "pgrep" },
            if is_pkill { "[-signal] " } else { "" }
        );
        return Err(1);
    }
    o.pattern = Some(pats[0]);
    Ok((o, pats))
}

fn pgrep_match(st: &PStat, cmd: &[u8], o: &PgrepOpts<'_>) -> bool {
    let pat = o.pattern.unwrap_or(b"");
    if let Some(u) = o.uid {
        match read_uids(st.pid) {
            Some((r, _)) if r == u => {}
            _ => return o.invert,
        }
    }
    if let Some(u) = o.euid {
        match read_uids(st.pid) {
            Some((_, e)) if e == u => {}
            _ => return o.invert,
        }
    }
    let mut hit = if o.full {
        if cmd.is_empty() {
            false
        } else {
            let mut spaced = cmd.to_vec();
            for c in spaced.iter_mut() {
                if *c == 0 {
                    *c = b' ';
                }
            }
            while spaced.last() == Some(&b' ') {
                spaced.pop();
            }
            if o.exact {
                spaced == pat
            } else {
                spaced.windows(pat.len().max(1)).any(|w| w == pat) || (pat.is_empty())
            }
        }
    } else if o.exact {
        st.comm == pat
    } else if pat.is_empty() {
        true
    } else {
        st.comm.windows(pat.len()).any(|w| w == pat)
    };
    if o.invert {
        hit = !hit;
    }
    hit
}

fn collect_matches(o: &PgrepOpts<'_>) -> Vec<(u32, Vec<u8>, u64)> {
    let mut out = Vec::new();
    let mut cmd = [0u8; 4096];
    let me = unsafe { libc::getpid() } as u32;
    for pid in proc_pids() {
        if pid == me {
            continue;
        }
        let st = match read_stat(pid) {
            Some(s) => s,
            None => continue,
        };
        let n = read_cmdline(pid, &mut cmd);
        if pgrep_match(&st, &cmd[..n], o) {
            out.push((pid, st.comm.clone(), st.starttime));
        }
    }
    if o.newest {
        out.sort_by_key(|t| t.2);
        let last = out.pop().map(|t| (t.0, t.1, t.2));
        out.clear();
        if let Some(t) = last {
            out.push(t);
        }
    } else if o.oldest {
        out.sort_by_key(|t| t.2);
        out.truncate(1);
    } else {
        out.sort_by_key(|t| t.0);
    }
    out
}

pub struct PgrepApplet;
impl Applet for PgrepApplet {
    fn name(&self) -> &'static str {
        "pgrep"
    }
    fn description(&self) -> &'static str {
        "Look up processes by name"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let (o, _) = match parse_pgrep(args, false) {
            Ok(v) => v,
            Err(rc) => return Ok(rc),
        };
        let m = collect_matches(&o);
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        if o.count {
            let mut line = Vec::with_capacity(16);
            push_u64(&mut line, m.len() as u64);
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            return Ok(if m.is_empty() { 1 } else { 0 });
        }
        for (pid, comm, _) in &m {
            let mut line = Vec::with_capacity(64);
            push_u64(&mut line, *pid as u64);
            if o.list {
                line.push(b' ');
                line.extend_from_slice(comm);
            }
            line.push(b'\n');
            out.write_all(&line)?;
        }
        out.flush()?;
        Ok(if m.is_empty() { 1 } else { 0 })
    }
}

pub struct PkillApplet;
impl Applet for PkillApplet {
    fn name(&self) -> &'static str {
        "pkill"
    }
    fn description(&self) -> &'static str {
        "Signal processes by name"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let (o, _) = match parse_pgrep(args, true) {
            Ok(v) => v,
            Err(rc) => return Ok(rc),
        };
        let m = collect_matches(&o);
        if m.is_empty() {
            return Ok(1);
        }
        let mut rc = 0;
        for (pid, _, _) in &m {
            if unsafe { libc::kill(*pid as i32, o.signal) } != 0
                && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
            {
                eprintln!("pkill: {}: {}", pid, std::io::Error::last_os_error());
                rc = 1;
            }
        }
        Ok(rc)
    }
}

pub struct PidofApplet;
impl Applet for PidofApplet {
    fn name(&self) -> &'static str {
        "pidof"
    }
    fn description(&self) -> &'static str {
        "Print PIDs of running processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut single = false;
        let mut omit: Vec<u32> = Vec::new();
        let mut names: Vec<&[u8]> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-s" {
                single = true;
            } else if b == b"-x" {
            } else if b == b"-o" {
                i += 1;
                if i >= args.len() {
                    eprintln!("pidof: -o needs a pid");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    Some(p) => omit.push(p),
                    None => {
                        eprintln!("pidof: invalid pid");
                        return Ok(1);
                    }
                }
            } else if b.len() > 1 && b[0] == b'-' {
                eprintln!("pidof: unknown option");
                return Ok(1);
            } else {
                names.push(b);
            }
            i += 1;
        }
        if names.is_empty() {
            eprintln!("usage: pidof [-s] [-o pid] name...");
            return Ok(1);
        }
        let mut found: Vec<u32> = Vec::new();
        let mut cmd = [0u8; 4096];
        for pid in proc_pids() {
            if omit.contains(&pid) {
                continue;
            }
            let st = match read_stat(pid) {
                Some(s) => s,
                None => continue,
            };
            let n = read_cmdline(pid, &mut cmd);
            let argv0base: Option<&[u8]> = if n > 0 {
                let end = cmd[..n].iter().position(|&c| c == 0).unwrap_or(n);
                let a0 = &cmd[..end];
                Some(match a0.iter().rposition(|&c| c == b'/') {
                    Some(p) => &a0[p + 1..],
                    None => a0,
                })
            } else {
                None
            };
            for want in &names {
                let wb = match want.iter().rposition(|&c| c == b'/') {
                    Some(p) => &want[p + 1..],
                    None => want,
                };
                if st.comm == *wb || argv0base == Some(wb) {
                    found.push(pid);
                    break;
                }
            }
        }
        if found.is_empty() {
            return Ok(1);
        }
        found.sort_unstable();
        if single {
            found.truncate(1);
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line = Vec::with_capacity(64);
        for (k, p) in found.iter().enumerate() {
            if k > 0 {
                line.push(b' ');
            }
            push_u64(&mut line, *p as u64);
        }
        line.push(b'\n');
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}

fn read_cpu_total() -> u64 {
    let mut buf = [0u8; 512];
    let n = read_small("/proc/stat", &mut buf);
    let b = &buf[..n];
    let mut i = 0;
    while i < b.len() && b[i] != b'\n' {
        i += 1;
    }

    let mut total: u64 = 0;
    let mut v: u64 = 0;
    let mut ind = false;
    for &c in &b[..i] {
        if c.is_ascii_digit() {
            v = v.saturating_mul(10).saturating_add((c - b'0') as u64);
            ind = true;
        } else if ind {
            total = total.saturating_add(v);
            v = 0;
            ind = false;
        }
    }
    total.saturating_add(v)
}

fn parse_delay(b: &[u8]) -> u64 {
    let mut secs: u64 = 0;
    let mut frac: u64 = 0;
    let mut div: u64 = 1;
    let mut after = false;
    for &c in b {
        if c.is_ascii_digit() {
            if after {
                if div < 1000 {
                    frac = frac * 10 + (c - b'0') as u64;
                    div *= 10;
                }
            } else {
                secs = secs.saturating_mul(10).saturating_add((c - b'0') as u64);
            }
        } else if c == b'.' {
            after = true;
        } else {
            break;
        }
    }
    secs.saturating_mul(1000)
        .saturating_add(frac * 1000 / div.max(1))
}

pub struct TopApplet;
#[allow(clippy::type_complexity)]
type TopRow = (u32, Vec<u8>, u8, f64, u64, u64, u64, Vec<u8>);
impl Applet for TopApplet {
    fn name(&self) -> &'static str {
        "top"
    }
    fn description(&self) -> &'static str {
        "Show running processes (batch snapshot)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut iters: u64 = 1;
        let mut delay_ms: u64 = 300;
        let mut delay_given = false;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-b" {
            } else if b == b"-n" {
                i += 1;
                if i >= args.len() {
                    eprintln!("top: -n needs a number");
                    return Ok(1);
                }
                iters = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(1);
            } else if b.starts_with(b"-n") && b.len() > 2 {
                iters = std::str::from_utf8(&b[2..])
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(1);
            } else if b == b"-d" {
                i += 1;
                if i >= args.len() {
                    eprintln!("top: -d needs a delay");
                    return Ok(1);
                }
                delay_ms = parse_delay(args[i].as_bytes());
                delay_given = true;
            } else if b.starts_with(b"-d") && b.len() > 2 {
                delay_ms = parse_delay(&b[2..]);
                delay_given = true;
            }
            i += 1;
        }
        if iters == 0 {
            iters = 1;
        }
        let sample_gap = if delay_given {
            delay_ms
        } else if iters == 1 {
            300
        } else {
            1000
        };
        let mut mem_total: u64 = 1;
        {
            let mut mb = [0u8; 4096];
            let n = read_small("/proc/meminfo", &mut mb);
            let b = &mb[..n];
            let mut k = 0;
            while k < b.len() {
                if b[k..].starts_with(b"MemTotal:") {
                    let mut j = k + 9;
                    while j < b.len() && !b[j].is_ascii_digit() {
                        j += 1;
                    }
                    mem_total = 0;
                    while j < b.len() && b[j].is_ascii_digit() {
                        mem_total = mem_total * 10 + (b[j] - b'0') as u64;
                        j += 1;
                    }
                    break;
                }
                while k < b.len() && b[k] != b'\n' {
                    k += 1;
                }
                k += 1;
            }
        }
        let page_kb = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64 / 1024;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for it in 0..iters {
            let t0 = read_cpu_total();
            let mut snap0: Vec<(u32, u64)> = Vec::new();
            for pid in proc_pids() {
                if let Some(s) = read_stat(pid) {
                    snap0.push((pid, s.utime + s.stime));
                }
            }
            if sample_gap > 0 {
                let ts = libc::timespec {
                    tv_sec: (sample_gap / 1000) as libc::time_t,
                    tv_nsec: ((sample_gap % 1000) * 1_000_000) as libc::c_long,
                };
                unsafe { libc::nanosleep(&ts, std::ptr::null_mut()) };
            }
            let t1 = read_cpu_total();
            let dt = t1.saturating_sub(t0).max(1);
            let mut rows: Vec<TopRow> = Vec::new();
            let mut cmd = [0u8; 512];
            for (pid, c0) in &snap0 {
                let st = match read_stat(*pid) {
                    Some(s) => s,
                    None => continue,
                };
                let dc = (st.utime + st.stime).saturating_sub(*c0);
                let pct = dc as f64 * 100.0 / dt as f64;
                let rss_kb = (st.rss_pages.max(0) as u64).saturating_mul(page_kb);
                let mempct = rss_kb as f64 * 100.0 / mem_total.max(1) as f64;
                let n = read_cmdline(*pid, &mut cmd);
                let disp = if n > 0 {
                    let end = cmd[..n].iter().position(|&c| c == 0).unwrap_or(n);
                    cmd[..end].to_vec()
                } else {
                    let mut v = Vec::with_capacity(st.comm.len() + 2);
                    v.push(b'[');
                    v.extend_from_slice(&st.comm);
                    v.push(b']');
                    v
                };
                let uid = read_uids(*pid).map(|t| t.0).unwrap_or(0);
                rows.push((
                    *pid,
                    user_name(uid),
                    st.state,
                    pct,
                    mempct as u64,
                    rss_kb,
                    st.vsize / 1024,
                    disp,
                ));
            }
            rows.sort_by(|a, b| {
                b.3.partial_cmp(&a.3)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.0.cmp(&b.0))
            });
            out.write_all(b"PID USER     S %CPU %MEM   VIRT   RES COMMAND\n")?;
            for (pid, user, state, pct, memp, res, virt, disp) in rows.iter() {
                let mut line = Vec::with_capacity(128);
                push_u64(&mut line, *pid as u64);
                line.push(b' ');
                let mut u = user.clone();
                u.truncate(8);
                line.extend_from_slice(&u);
                while line.len() < 14 {
                    line.push(b' ');
                }
                line.push(*state);
                line.push(b' ');
                let c10 = (*pct * 10.0) as u64;
                push_u64(&mut line, c10 / 10);
                line.push(b'.');
                line.push(b'0' + (c10 % 10) as u8);
                line.push(b' ');
                push_u64(&mut line, *memp);
                line.push(b' ');
                push_u64(&mut line, *virt);
                line.push(b' ');
                push_u64(&mut line, *res);
                line.push(b' ');
                let mut d = disp.clone();
                d.truncate(64);
                line.extend_from_slice(&d);
                line.push(b'\n');
                out.write_all(&line)?;
            }
            out.flush()?;
            if it + 1 < iters && delay_ms > 0 {
                let ts = libc::timespec {
                    tv_sec: (delay_ms / 1000) as libc::time_t,
                    tv_nsec: ((delay_ms % 1000) * 1_000_000) as libc::c_long,
                };
                unsafe { libc::nanosleep(&ts, std::ptr::null_mut()) };
            }
        }
        Ok(0)
    }
}

pub struct PmapApplet;
impl Applet for PmapApplet {
    fn name(&self) -> &'static str {
        "pmap"
    }
    fn description(&self) -> &'static str {
        "Report memory map of processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut ext = false;
        let mut pids: Vec<u32> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b == b"-x" {
                ext = true;
            } else if b == b"-q" {
            } else if b.first() == Some(&b'-') {
                eprintln!("pmap: unknown option");
                return Ok(1);
            } else {
                match std::str::from_utf8(b)
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    Some(p) => pids.push(p),
                    None => {
                        eprintln!("pmap: invalid pid");
                        return Ok(1);
                    }
                }
            }
        }
        if pids.is_empty() {
            eprintln!("usage: pmap [-x] pid...");
            return Ok(1);
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut rc = 0;
        for pid in pids {
            let mut path = [0u8; 48];
            let n = proc_path(pid, b"/maps", &mut path);
            let ps = match std::str::from_utf8(&path[..n]) {
                Ok(s) => s.to_string(),
                Err(_) => continue,
            };
            let data = match std::fs::read(&ps) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("pmap: {}: {}", pid, e);
                    rc = 1;
                    continue;
                }
            };
            let mut line = Vec::with_capacity(64);
            push_u64(&mut line, pid as u64);
            line.extend_from_slice(b": maps\n");
            out.write_all(&line)?;
            out.write_all(&data)?;

            let mut total: u64 = 0;
            for ln in data.split(|&c| c == b'\n') {
                if let Some(dash) = ln.iter().position(|&c| c == b'-') {
                    let mut sp = dash;
                    while sp < ln.len() && ln[sp] != b' ' {
                        sp += 1;
                    }
                    let lo =
                        u64::from_str_radix(std::str::from_utf8(&ln[..dash]).unwrap_or(""), 16)
                            .unwrap_or(0);
                    let hi = u64::from_str_radix(
                        std::str::from_utf8(&ln[dash + 1..sp]).unwrap_or(""),
                        16,
                    )
                    .unwrap_or(0);
                    total = total.saturating_add(hi.saturating_sub(lo));
                }
            }
            if ext {
                let mut spath = [0u8; 48];
                let sn = proc_path(pid, b"/smaps", &mut spath);
                let mut rss: u64 = 0;
                let mut pss: u64 = 0;
                if let Ok(ss) = std::str::from_utf8(&spath[..sn]) {
                    let mut sb = [0u8; 65536];
                    let rn = read_small(ss, &mut sb);
                    let mut k = 0;
                    while k < rn {
                        let mut e = k;
                        while e < rn && sb[e] != b'\n' {
                            e += 1;
                        }
                        let l = &sb[k..e];
                        if l.starts_with(b"Rss:") || l.starts_with(b"Pss:") {
                            let mut j = 4;
                            while j < l.len() && !l[j].is_ascii_digit() {
                                j += 1;
                            }
                            let mut v: u64 = 0;
                            while j < l.len() && l[j].is_ascii_digit() {
                                v = v * 10 + (l[j] - b'0') as u64;
                                j += 1;
                            }
                            if l.starts_with(b"Rss:") {
                                rss += v;
                            } else {
                                pss += v;
                            }
                        }
                        k = e + 1;
                    }
                }
                let mut tl = Vec::with_capacity(64);
                tl.extend_from_slice(b"total ");
                push_u64(&mut tl, total / 1024);
                tl.extend_from_slice(b"K rss ");
                push_u64(&mut tl, rss);
                tl.extend_from_slice(b"K pss ");
                push_u64(&mut tl, pss);
                tl.extend_from_slice(b"K\n");
                out.write_all(&tl)?;
            } else {
                let mut tl = Vec::with_capacity(32);
                tl.extend_from_slice(b"total ");
                push_u64(&mut tl, total / 1024);
                tl.extend_from_slice(b"K\n");
                out.write_all(&tl)?;
            }
        }
        out.flush()?;
        Ok(rc)
    }
}

pub struct PwdxApplet;
impl Applet for PwdxApplet {
    fn name(&self) -> &'static str {
        "pwdx"
    }
    fn description(&self) -> &'static str {
        "Report current working directory of processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("usage: pwdx pid...");
            return Ok(1);
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut rc = 0;
        for a in args {
            let pid: u32 = match std::str::from_utf8(a.as_bytes())
                .ok()
                .and_then(|s| s.parse().ok())
            {
                Some(p) => p,
                None => {
                    eprintln!("pwdx: invalid pid");
                    rc = 1;
                    continue;
                }
            };
            let mut path = [0u8; 48];
            let n = proc_path(pid, b"/cwd", &mut path);
            let ps = match std::str::from_utf8(&path[..n]) {
                Ok(s) => s,
                Err(_) => {
                    rc = 1;
                    continue;
                }
            };
            match std::fs::read_link(ps) {
                Ok(target) => {
                    let mut line = Vec::with_capacity(64);
                    push_u64(&mut line, pid as u64);
                    line.extend_from_slice(b": ");
                    line.extend_from_slice(target.as_os_str().as_bytes());
                    line.push(b'\n');
                    out.write_all(&line)?;
                }
                Err(e) => {
                    eprintln!("pwdx: {}: {}", pid, e);
                    rc = 1;
                }
            }
        }
        out.flush()?;
        Ok(rc)
    }
}

fn meminfo_kb(key: &[u8]) -> u64 {
    let mut buf = [0u8; 8192];
    let n = read_small("/proc/meminfo", &mut buf);
    let b = &buf[..n];
    let mut i = 0;
    while i + key.len() + 1 < b.len() {
        if &b[i..i + key.len()] == key && b[i + key.len()] == b':' {
            let mut j = i + key.len() + 1;
            while j < b.len() && !b[j].is_ascii_digit() {
                j += 1;
            }
            let mut v: u64 = 0;
            while j < b.len() && b[j].is_ascii_digit() {
                v = v * 10 + (b[j] - b'0') as u64;
                j += 1;
            }
            return v;
        }
        while i < b.len() && b[i] != b'\n' {
            i += 1;
        }
        i += 1;
    }
    0
}

fn vmstat_val(key: &[u8]) -> u64 {
    let mut buf = [0u8; 16384];
    let n = read_small("/proc/vmstat", &mut buf);
    let b = &buf[..n];
    let mut i = 0;
    while i < b.len() {
        if b[i..].starts_with(key) && b.get(i + key.len()) == Some(&b' ') {
            let mut j = i + key.len() + 1;
            let mut v: u64 = 0;
            while j < b.len() && b[j].is_ascii_digit() {
                v = v * 10 + (b[j] - b'0') as u64;
                j += 1;
            }
            return v;
        }
        while i < b.len() && b[i] != b'\n' {
            i += 1;
        }
        i += 1;
    }
    0
}

fn trailing_num(ln: &[u8]) -> u64 {
    let mut v = 0u64;
    let mut m = 1u64;
    for &c in ln.iter().rev() {
        if !c.is_ascii_digit() {
            break;
        }
        v += (c - b'0') as u64 * m;
        m *= 10;
    }
    v
}

fn vmstat_line(out: &mut dyn Write) -> std::io::Result<()> {
    let mut sb = [0u8; 1024];
    let n = read_small("/proc/stat", &mut sb);
    let mut r = 0;
    let mut bl = 0;
    let mut cpu = [0u64; 8];
    for ln in sb[..n].split(|&c| c == b'\n') {
        if ln.starts_with(b"procs_running") {
            r = trailing_num(ln);
        } else if ln.starts_with(b"procs_blocked") {
            bl = trailing_num(ln);
        } else if ln.starts_with(b"cpu ") {
            let mut k = 4;
            let mut f = 0;
            while k < ln.len() && f < 8 {
                while k < ln.len() && ln[k] == b' ' {
                    k += 1;
                }
                let mut v: u64 = 0;
                let mut any = false;
                while k < ln.len() && ln[k].is_ascii_digit() {
                    v = v * 10 + (ln[k] - b'0') as u64;
                    k += 1;
                    any = true;
                }
                if any {
                    cpu[f] = v;
                    f += 1;
                } else {
                    break;
                }
            }
        }
    }
    let total: u64 = cpu.iter().sum();
    let idle = cpu[3] + cpu[4];
    let mut line = Vec::with_capacity(128);
    for v in [
        r,
        bl,
        meminfo_kb(b"SwapTotal") - meminfo_kb(b"SwapFree"),
        meminfo_kb(b"MemFree"),
        meminfo_kb(b"Buffers"),
        meminfo_kb(b"Cached"),
        vmstat_val(b"pswpin"),
        vmstat_val(b"pswpout"),
        vmstat_val(b"pgpgin"),
        vmstat_val(b"pgpgout"),
        vmstat_val(b"intr"),
        vmstat_val(b"ctxt"),
    ] {
        push_u64(&mut line, v);
        line.push(b' ');
    }
    let us = cpu[0]
        .saturating_add(cpu[1])
        .saturating_add(cpu[2])
        .saturating_mul(100)
        .checked_div(total)
        .unwrap_or(0);
    let sy = (cpu[5] + cpu[6] + cpu[7]).checked_div(total).unwrap_or(0);
    let id = idle.saturating_mul(100).checked_div(total).unwrap_or(0);
    push_u64(&mut line, us);
    line.push(b' ');
    push_u64(&mut line, sy);
    line.push(b' ');
    push_u64(&mut line, id);
    line.push(b'\n');
    out.write_all(&line)
}

pub struct VmstatApplet;
impl Applet for VmstatApplet {
    fn name(&self) -> &'static str {
        "vmstat"
    }
    fn description(&self) -> &'static str {
        "Report virtual memory statistics"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.iter().any(|a| a.as_bytes() == b"-s") {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            for (k, key) in [
                b"MemTotal".as_slice(),
                b"MemFree".as_slice(),
                b"Buffers".as_slice(),
                b"Cached".as_slice(),
                b"SwapTotal".as_slice(),
                b"SwapFree".as_slice(),
            ]
            .iter()
            .enumerate()
            {
                let _ = k;
                let mut line = Vec::with_capacity(48);
                push_u64(&mut line, meminfo_kb(key));
                line.extend_from_slice(b" kB ");
                line.extend_from_slice(key);
                line.push(b'\n');
                out.write_all(&line)?;
            }
            out.flush()?;
            return Ok(0);
        }
        let mut delay_ms = 0;
        let mut count: u64 = 1;
        for (pos, a) in args.iter().enumerate() {
            if a.as_bytes().first() == Some(&b'-') {
                eprintln!("vmstat: unknown option");
                return Ok(1);
            }
            if pos == 0 {
                delay_ms = parse_delay(a.as_bytes());
            } else if pos == 1 {
                count = std::str::from_utf8(a.as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
            }
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        out.write_all(
            b"procs -----------memory---------- ---swap-- -----io---- -system-- ------cpu-----\n",
        )?;
        out.write_all(
            b" r  b   swpd   free   buff  cache   si   so    bi    bo   in   cs us sy id\n",
        )?;
        if count == 0 {
            count = u64::MAX / 2;
        }
        for it in 0..count {
            vmstat_line(&mut out)?;
            out.flush()?;
            if it + 1 < count && delay_ms > 0 {
                let ts = libc::timespec {
                    tv_sec: (delay_ms / 1000) as libc::time_t,
                    tv_nsec: ((delay_ms % 1000) * 1_000_000) as libc::c_long,
                };
                unsafe { libc::nanosleep(&ts, std::ptr::null_mut()) };
            }
        }
        Ok(0)
    }
}

pub struct Killall5Applet;
impl Applet for Killall5Applet {
    fn name(&self) -> &'static str {
        "killall5"
    }
    fn description(&self) -> &'static str {
        "Signal all processes except caller"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sig = libc::SIGTERM;
        let mut omit: Vec<u32> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-o" {
                i += 1;
                if i >= args.len() {
                    eprintln!("killall5: -o needs a pid");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    Some(p) => omit.push(p),
                    None => {
                        eprintln!("killall5: invalid pid");
                        return Ok(1);
                    }
                }
            } else if b == b"-s" {
                i += 1;
                if i >= args.len() {
                    eprintln!("killall5: -s needs a signal");
                    return Ok(1);
                }
                match sig_from_name(args[i].as_bytes()) {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("killall5: unknown signal");
                        return Ok(1);
                    }
                }
            } else if b.len() > 1 && b[0] == b'-' && !b[1].is_ascii_digit() {
                match sig_from_name(&b[1..]) {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("killall5: unknown signal");
                        return Ok(1);
                    }
                }
            } else if b.len() > 1 && b[0] == b'-' {
                match std::str::from_utf8(&b[1..])
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("killall5: unknown signal");
                        return Ok(1);
                    }
                }
            } else {
                eprintln!("usage: killall5 [-SIGNAL] [-o pid] [-s sig]");
                return Ok(1);
            }
            i += 1;
        }
        let me = unsafe { libc::getpid() } as u32;
        let sid = unsafe { libc::getsid(0) } as u32;

        let mut rc = 2;

        if sig != libc::SIGSTOP && sig != libc::SIGCONT {
            unsafe { libc::kill(-1, libc::SIGSTOP) };
        }
        for pid in proc_pids() {
            if pid == me || pid == 1 || omit.contains(&pid) {
                continue;
            }

            let psid = unsafe { libc::getsid(pid as i32) } as u32;
            if psid == sid {
                continue;
            }
            unsafe { libc::kill(pid as i32, sig) };
            rc = 0;
        }
        if sig != libc::SIGSTOP && sig != libc::SIGCONT {
            unsafe { libc::kill(-1, libc::SIGCONT) };
        }
        Ok(rc)
    }
}

pub struct ReniceApplet;
impl Applet for ReniceApplet {
    fn name(&self) -> &'static str {
        "renice"
    }
    fn description(&self) -> &'static str {
        "Alter priority of running processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut which = libc::PRIO_PROCESS;
        let mut prio: Option<i32> = None;
        let mut ids: Vec<i32> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-n" {
                i += 1;
                if i >= args.len() {
                    eprintln!("renice: -n needs a priority");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(p) => prio = Some(p),
                    None => {
                        eprintln!("renice: invalid priority");
                        return Ok(1);
                    }
                }
            } else if b == b"-g" {
                which = libc::PRIO_PGRP;
            } else if b == b"-p" {
                which = libc::PRIO_PROCESS;
            } else if b == b"-u" {
                which = libc::PRIO_USER;
            } else if b.len() > 1
                && b[0] == b'-'
                && (b[1].is_ascii_digit() || b[1] == b'+' || b[1] == b'-')
            {
                match std::str::from_utf8(b)
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(p) => prio = Some(p),
                    None => {
                        eprintln!("renice: invalid priority");
                        return Ok(1);
                    }
                }
            } else if prio.is_none() && (b[0].is_ascii_digit() || b[0] == b'+' || b[0] == b'-') {
                match std::str::from_utf8(b)
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(p) => prio = Some(p),
                    None => {
                        eprintln!("renice: invalid priority");
                        return Ok(1);
                    }
                }
            } else {
                if which == libc::PRIO_USER {
                    match uid_of_name(b) {
                        Some(u) => ids.push(u as i32),
                        None => {
                            eprintln!("renice: unknown user");
                            return Ok(1);
                        }
                    }
                } else {
                    match std::str::from_utf8(b)
                        .ok()
                        .and_then(|s| s.parse::<i32>().ok())
                    {
                        Some(p) => ids.push(p),
                        None => {
                            eprintln!("renice: invalid id");
                            return Ok(1);
                        }
                    }
                }
            }
            i += 1;
        }
        let prio = match prio {
            Some(p) => p,
            None => {
                eprintln!("usage: renice [-n] prio [-g|-p|-u] id...");
                return Ok(1);
            }
        };
        if ids.is_empty() {
            eprintln!("usage: renice [-n] prio [-g|-p|-u] id...");
            return Ok(1);
        }
        let mut rc = 0;
        for id in ids {
            if unsafe { libc::setpriority(which, id as u32, prio) } != 0 {
                eprintln!("renice: {}: {}", id, std::io::Error::last_os_error());
                rc = 1;
            }
        }
        Ok(rc)
    }
}

pub struct IoniceApplet;
impl Applet for IoniceApplet {
    fn name(&self) -> &'static str {
        "ionice"
    }
    fn description(&self) -> &'static str {
        "Set or get I/O scheduling class and priority"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut class: Option<i32> = None;
        let mut level: Option<i32> = None;
        let mut pid: i32 = 0;
        let mut prog: Option<usize> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-c" {
                i += 1;
                if i >= args.len() {
                    eprintln!("ionice: -c needs a class");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(c @ 0..=3) => class = Some(c),
                    _ => {
                        eprintln!("ionice: bad class (0-3)");
                        return Ok(1);
                    }
                }
            } else if b.len() == 3 && b[..2] == *b"-c" && b[2].is_ascii_digit() {
                let c = (b[2] - b'0') as i32;
                if c > 3 {
                    eprintln!("ionice: bad class (0-3)");
                    return Ok(1);
                }
                class = Some(c);
            } else if b == b"-n" {
                i += 1;
                if i >= args.len() {
                    eprintln!("ionice: -n needs a level");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(n @ 0..=7) => level = Some(n),
                    _ => {
                        eprintln!("ionice: bad level (0-7)");
                        return Ok(1);
                    }
                }
            } else if b.len() == 3 && b[..2] == *b"-n" && b[2].is_ascii_digit() {
                let n = (b[2] - b'0') as i32;
                if n > 7 {
                    eprintln!("ionice: bad level (0-7)");
                    return Ok(1);
                }
                level = Some(n);
            } else if b == b"-p" {
                i += 1;
                if i >= args.len() {
                    eprintln!("ionice: -p needs a pid");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(p) => pid = p,
                    None => {
                        eprintln!("ionice: invalid pid");
                        return Ok(1);
                    }
                }
            } else if b.first() == Some(&b'-') {
                eprintln!("ionice: unknown option");
                return Ok(1);
            } else {
                prog = Some(i);
                break;
            }
            i += 1;
        }
        unsafe extern "C" {
            fn syscall(num: libc::c_long, ...) -> libc::c_long;
        }
        const IOPRIO_CLASS_SHIFT: i32 = 13;
        if let Some(pi) = prog {
            if class.is_some() || level.is_some() {
                let c = class.unwrap_or(2);
                let n = level.unwrap_or(4);
                let v = ((c << IOPRIO_CLASS_SHIFT) | n) as libc::c_long;
                let r = unsafe { syscall(libc::SYS_ioprio_set as libc::c_long, 1, 0, v) };
                if r != 0 {
                    eprintln!("ionice: {}", std::io::Error::last_os_error());
                    return Ok(1);
                }
            }
            let cmd: Vec<OsString> = args[pi..].to_vec();
            return Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]));
        }
        if class.is_none() && level.is_none() {
            let r = unsafe { syscall(libc::SYS_ioprio_get as libc::c_long, 1, pid) };
            if r < 0 {
                eprintln!("ionice: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            let c = (r >> IOPRIO_CLASS_SHIFT) as i32;
            let n = (r & 0xff) as i32;
            let cname = match c {
                0 => "none",
                1 => "realtime",
                2 => "best-effort",
                3 => "idle",
                _ => "unknown",
            };
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            let mut line = Vec::with_capacity(48);
            line.extend_from_slice(cname.as_bytes());
            line.extend_from_slice(b": prio ");
            push_u64(&mut line, n as u64);
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            return Ok(0);
        }
        let c = class.unwrap_or(2);
        let n = level.unwrap_or(4);
        let v = ((c << IOPRIO_CLASS_SHIFT) | n) as libc::c_long;
        let r = unsafe { syscall(libc::SYS_ioprio_set as libc::c_long, 1, pid, v) };
        if r != 0 {
            eprintln!("ionice: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct ChrtApplet;
impl Applet for ChrtApplet {
    fn name(&self) -> &'static str {
        "chrt"
    }
    fn description(&self) -> &'static str {
        "Get or set real-time scheduling attributes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut pid_mode = false;
        let mut policy: Option<i32> = None;
        let prio: i32;
        let mut max = false;
        let mut rest: Vec<OsString> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b == b"-p" {
                pid_mode = true;
            } else if b == b"-f" {
                policy = Some(libc::SCHED_FIFO);
            } else if b == b"-r" {
                policy = Some(libc::SCHED_RR);
            } else if b == b"-o" {
                policy = Some(libc::SCHED_OTHER);
            } else if b == b"-b" {
                policy = Some(libc::SCHED_BATCH);
            } else if b == b"-i" {
                policy = Some(libc::SCHED_IDLE);
            } else if b == b"-m" {
                max = true;
            } else if b.first() == Some(&b'-') {
                eprintln!("chrt: unknown option");
                return Ok(1);
            } else {
                rest.push(a.clone());
            }
        }
        if max {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            for (pol, nm) in [
                (libc::SCHED_FIFO, "SCHED_FIFO"),
                (libc::SCHED_RR, "SCHED_RR"),
                (libc::SCHED_BATCH, "SCHED_BATCH"),
                (libc::SCHED_IDLE, "SCHED_IDLE"),
                (libc::SCHED_OTHER, "SCHED_OTHER"),
            ] {
                let mn = unsafe { libc::sched_get_priority_min(pol) };
                let mx = unsafe { libc::sched_get_priority_max(pol) };
                let mut line = Vec::with_capacity(48);
                line.extend_from_slice(nm.as_bytes());
                line.extend_from_slice(b" min/max priority\t: ");
                push_u64(&mut line, mn.max(0) as u64);
                line.push(b'/');
                push_u64(&mut line, mx.max(0) as u64);
                line.push(b'\n');
                out.write_all(&line)?;
            }
            out.flush()?;
            return Ok(0);
        }
        let show = |pid: i32| -> Result<i32> {
            let p = unsafe { libc::sched_getscheduler(pid) };
            if p < 0 {
                eprintln!("chrt: {}: {}", pid, std::io::Error::last_os_error());
                return Ok(1);
            }
            let mut sp: libc::sched_param = unsafe { std::mem::zeroed() };
            if unsafe { libc::sched_getparam(pid, &mut sp) } != 0 {
                eprintln!("chrt: {}: {}", pid, std::io::Error::last_os_error());
                return Ok(1);
            }
            let nm = match p {
                libc::SCHED_FIFO => "SCHED_FIFO",
                libc::SCHED_RR => "SCHED_RR",
                libc::SCHED_BATCH => "SCHED_BATCH",
                libc::SCHED_IDLE => "SCHED_IDLE",
                _ => "SCHED_OTHER",
            };
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            let mut line = Vec::with_capacity(64);
            line.extend_from_slice(b"pid ");
            push_u64(&mut line, pid as u64);
            line.extend_from_slice(b"'s current scheduling policy: ");
            line.extend_from_slice(nm.as_bytes());
            line.push(b'\n');
            out.write_all(&line)?;
            let mut l2 = Vec::with_capacity(64);
            l2.extend_from_slice(b"pid ");
            push_u64(&mut l2, pid as u64);
            l2.extend_from_slice(b"'s current scheduling priority: ");
            push_u64(&mut l2, sp.sched_priority.max(0) as u64);
            l2.push(b'\n');
            out.write_all(&l2)?;
            out.flush()?;
            Ok(0)
        };
        if pid_mode {
            let mut nums: Vec<i32> = Vec::new();
            for r in &rest {
                match std::str::from_utf8(r.as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(n) => nums.push(n),
                    None => {
                        eprintln!("usage: chrt -p [-f|-r|-o|-b|-i] [prio] pid");
                        return Ok(1);
                    }
                }
            }
            if policy.is_none() && nums.len() == 1 {
                return show(nums[0]);
            }
            let (pr, target) = match nums.len() {
                1 => (0, nums[0]),
                2 => (nums[0], nums[1]),
                _ => {
                    eprintln!("usage: chrt -p [-f|-r|-o|-b|-i] [prio] pid");
                    return Ok(1);
                }
            };
            let pol = policy.unwrap_or(libc::SCHED_RR);
            let mut sp: libc::sched_param = unsafe { std::mem::zeroed() };
            sp.sched_priority = pr;
            if unsafe { libc::sched_setscheduler(target, pol, &sp) } != 0 {
                eprintln!("chrt: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            return Ok(0);
        }
        if rest.is_empty() {
            eprintln!("usage: chrt [-p pid] [-f|-r|-o|-b|-i] [prio] pid|cmd...");
            return Ok(1);
        }

        let pol = policy.unwrap_or(libc::SCHED_OTHER);
        let first_num = std::str::from_utf8(rest[0].as_bytes())
            .ok()
            .and_then(|s| s.parse::<i32>().ok());
        if rest.len() >= 2 && first_num.is_some() {
            prio = first_num.unwrap_or(0);

            if rest.len() == 2
                && std::str::from_utf8(rest[1].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                    .is_some()
            {
                let target: i32 = std::str::from_utf8(rest[1].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let mut sp: libc::sched_param = unsafe { std::mem::zeroed() };
                sp.sched_priority = prio;
                if unsafe { libc::sched_setscheduler(target, pol, &sp) } != 0 {
                    eprintln!("chrt: {}", std::io::Error::last_os_error());
                    return Ok(1);
                }
                return Ok(0);
            }
            let mut sp: libc::sched_param = unsafe { std::mem::zeroed() };
            sp.sched_priority = prio;
            if unsafe { libc::sched_setscheduler(0, pol, &sp) } != 0 {
                eprintln!("chrt: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            let cmd: Vec<OsString> = rest[1..].to_vec();
            return Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]));
        }

        if rest.len() == 1
            && std::str::from_utf8(rest[0].as_bytes())
                .ok()
                .and_then(|s| s.parse::<i32>().ok())
                .is_some()
        {
            let pid: i32 = std::str::from_utf8(rest[0].as_bytes())
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            return show(pid);
        }
        eprintln!("usage: chrt [-p pid] [-f|-r|-o|-b|-i] [prio] pid|cmd...");
        Ok(1)
    }
}

pub struct TasksetApplet;
impl Applet for TasksetApplet {
    fn name(&self) -> &'static str {
        "taskset"
    }
    fn description(&self) -> &'static str {
        "Get or set CPU affinity"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut show_pid_only = false;
        let mut rest: Vec<OsString> = Vec::new();
        for a in args {
            if a.as_bytes() == b"-p" {
                show_pid_only = true;
            } else {
                rest.push(a.clone());
            }
        }
        let show = |pid: i32| -> Result<i32> {
            let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
            let sz = std::mem::size_of::<libc::cpu_set_t>();
            if unsafe { libc::sched_getaffinity(pid, sz, &mut set) } != 0 {
                eprintln!("taskset: {}: {}", pid, std::io::Error::last_os_error());
                return Ok(1);
            }
            let words: &[u64] =
                unsafe { std::slice::from_raw_parts((&raw const set) as *const u64, sz / 8) };
            let mut mask: u64 = 0;
            for (k, w) in words.iter().enumerate().take(1) {
                let _ = k;
                mask = *w;
            }
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            let mut line = Vec::with_capacity(48);
            line.extend_from_slice(b"pid ");
            push_u64(&mut line, pid as u64);
            line.extend_from_slice(b"'s current affinity mask: ");
            let mut hex = [0u8; 16];
            let mut hn = 0;
            let mut v = mask;
            if v == 0 {
                hex[0] = b'0';
                hn = 1;
            } else {
                let mut rev = [0u8; 16];
                let mut rn = 0;
                while v > 0 {
                    let d = (v & 0xf) as u8;
                    rev[rn] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
                    v >>= 4;
                    rn += 1;
                }
                while rn > 0 {
                    rn -= 1;
                    hex[hn] = rev[rn];
                    hn += 1;
                }
            }
            line.extend_from_slice(&hex[..hn]);
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            Ok(0)
        };
        let parse_mask = |b: &[u8]| -> Option<u64> {
            let h = if b.starts_with(b"0x") || b.starts_with(b"0X") {
                &b[2..]
            } else {
                b
            };

            let g = match h.iter().rposition(|&c| c == b',') {
                Some(p) => &h[p + 1..],
                None => h,
            };
            if g.is_empty() || !g.iter().all(|c| c.is_ascii_hexdigit()) {
                return None;
            }
            u64::from_str_radix(std::str::from_utf8(g).ok()?, 16).ok()
        };
        match rest.len() {
            0 => {
                eprintln!("usage: taskset [-p] [mask] pid|cmd...");
                Ok(1)
            }
            1 => {
                if show_pid_only {
                    match std::str::from_utf8(rest[0].as_bytes())
                        .ok()
                        .and_then(|s| s.parse::<i32>().ok())
                    {
                        Some(pid) => show(pid),
                        None => {
                            eprintln!("taskset: invalid pid");
                            Ok(1)
                        }
                    }
                } else {
                    if let Some(pid) = std::str::from_utf8(rest[0].as_bytes())
                        .ok()
                        .and_then(|s| s.parse::<i32>().ok())
                    {
                        show(pid)
                    } else {
                        eprintln!("usage: taskset [-p] [mask] pid|cmd...");
                        Ok(1)
                    }
                }
            }
            _ => {
                let mask = match parse_mask(rest[0].as_bytes()) {
                    Some(m) => m,
                    None => {
                        eprintln!("taskset: invalid mask");
                        return Ok(1);
                    }
                };

                if rest.len() == 2
                    && std::str::from_utf8(rest[1].as_bytes())
                        .ok()
                        .and_then(|s| s.parse::<i32>().ok())
                        .is_some()
                {
                    let pid: i32 = std::str::from_utf8(rest[1].as_bytes())
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    let sz = std::mem::size_of::<libc::cpu_set_t>();
                    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
                    unsafe {
                        let words =
                            std::slice::from_raw_parts_mut((&raw mut set) as *mut u64, sz / 8);
                        for w in words.iter_mut() {
                            *w = 0;
                        }
                        words[0] = mask;
                        if libc::sched_setaffinity(pid, sz, &set) != 0 {
                            eprintln!("taskset: {}", std::io::Error::last_os_error());
                            return Ok(1);
                        }
                    }
                    return show(pid);
                }

                let sz = std::mem::size_of::<libc::cpu_set_t>();
                let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
                unsafe {
                    let words = std::slice::from_raw_parts_mut((&raw mut set) as *mut u64, sz / 8);
                    for w in words.iter_mut() {
                        *w = 0;
                    }
                    words[0] = mask;
                    if libc::sched_setaffinity(0, sz, &set) != 0 {
                        eprintln!("taskset: {}", std::io::Error::last_os_error());
                        return Ok(1);
                    }
                }
                let cmd: Vec<OsString> = rest[1..].to_vec();
                Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
            }
        }
    }
}

pub struct SetsidApplet;
impl Applet for SetsidApplet {
    fn name(&self) -> &'static str {
        "setsid"
    }
    fn description(&self) -> &'static str {
        "Run a program in a new session"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut prog_at: Option<usize> = None;
        for (k, a) in args.iter().enumerate() {
            if a.as_bytes() == b"-c" || a.as_bytes() == b"-w" {
                continue;
            } else if a.as_bytes().first() == Some(&b'-') {
                eprintln!("setsid: unknown option");
                return Ok(1);
            } else {
                prog_at = Some(k);
                break;
            }
        }

        unsafe {
            if libc::getpgrp() == libc::getpid() {
                let pid = libc::fork();
                if pid < 0 {
                    eprintln!("setsid: {}", std::io::Error::last_os_error());
                    return Ok(1);
                }
                if pid != 0 {
                    libc::_exit(0);
                }
            }
            if libc::setsid() < 0 {
                eprintln!("setsid: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
        }
        match prog_at {
            Some(p) => {
                let cmd: Vec<OsString> = args[p..].to_vec();
                Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
            }
            None => Ok(0),
        }
    }
}

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

pub struct ChrootApplet;
impl Applet for ChrootApplet {
    fn name(&self) -> &'static str {
        "chroot"
    }
    fn description(&self) -> &'static str {
        "Run command with a different root directory"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut root: Option<&[u8]> = None;
        let mut at: Option<usize> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--userspec" {
                i += 1;
            } else if b.starts_with(b"--userspec=") {
            } else if b.first() == Some(&b'-') {
                eprintln!("chroot: unknown option");
                return Ok(1);
            } else if root.is_none() {
                root = Some(b);
            } else {
                at = Some(i);
                break;
            }
            i += 1;
        }
        let root = match root {
            Some(r) => r,
            None => {
                eprintln!("usage: chroot NEWROOT [COMMAND...]");
                return Ok(1);
            }
        };
        use std::ffi::CString;
        let rc = CString::new(root).unwrap_or_else(|_| CString::new("/").unwrap());
        if unsafe { libc::chroot(rc.as_ptr()) } != 0 {
            eprintln!(
                "chroot: can't change root directory to '{}': {}",
                String::from_utf8_lossy(root),
                strerror_last()
            );
            return Ok(1);
        }
        if unsafe { libc::chdir(c"/".as_ptr()) } != 0 {
            eprintln!("chroot: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        match at {
            Some(p) => {
                let cmd: Vec<OsString> = args[p..].to_vec();
                Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
            }
            None => {
                let sh = OsString::from_vec(b"/bin/sh".to_vec());
                let dash = OsString::from_vec(b"-i".to_vec());
                Ok(exec_prog(b"/bin/sh", &[dash, sh]))
            }
        }
    }
}

pub struct CttyhackApplet;
impl Applet for CttyhackApplet {
    fn name(&self) -> &'static str {
        "cttyhack"
    }
    fn description(&self) -> &'static str {
        "Give a shell a controlling terminal"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut at: Option<usize> = None;
        for (k, a) in args.iter().enumerate() {
            if a.as_bytes() == b"-v" {
                continue;
            } else if a.as_bytes().first() == Some(&b'-') {
                eprintln!("cttyhack: unknown option");
                return Ok(1);
            } else {
                at = Some(k);
                break;
            }
        }
        let p = match at {
            Some(v) => v,
            None => {
                eprintln!("usage: cttyhack PROG [ARGS...]");
                return Ok(1);
            }
        };
        unsafe {
            if libc::getpgrp() == libc::getpid() {
                let pid = libc::fork();
                if pid < 0 {
                    eprintln!("cttyhack: {}", std::io::Error::last_os_error());
                    return Ok(1);
                }
                if pid != 0 {
                    let mut st = 0;
                    libc::waitpid(pid, &mut st, 0);
                    if libc::WIFEXITED(st) {
                        return Ok(libc::WEXITSTATUS(st));
                    }
                    return Ok(1);
                }
            }
            if libc::setsid() < 0 {
                eprintln!("cttyhack: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            for fd in 0..3 {
                libc::ioctl(fd, libc::TIOCSCTTY as libc::c_ulong, 1);
            }
        }
        let cmd: Vec<OsString> = args[p..].to_vec();
        Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
    }
}
