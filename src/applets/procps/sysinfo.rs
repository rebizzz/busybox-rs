use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;

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

fn meminfo_val(buf: &[u8], key: &[u8]) -> u64 {
    let mut i = 0;
    while i + key.len() + 1 < buf.len() {
        if &buf[i..i + key.len()] == key && buf[i + key.len()] == b':' {
            let mut j = i + key.len() + 1;
            while j < buf.len() && (buf[j] == b' ' || buf[j] == b'\t') {
                j += 1;
            }
            let mut v: u64 = 0;
            while j < buf.len() && buf[j].is_ascii_digit() {
                v = v.saturating_mul(10).saturating_add((buf[j] - b'0') as u64);
                j += 1;
            }
            return v;
        }
        while i < buf.len() && buf[i] != b'\n' {
            i += 1;
        }
        i += 1;
    }
    0
}

struct ProcInfo<'a> {
    pid: u32,
    comm: &'a [u8],
    state: u8,
}

fn for_each_proc<F: FnMut(ProcInfo<'_>)>(scratch: &mut [u8; 512], mut f: F) {
    let dir = match std::fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return,
    };
    let mut path = [0u8; 32];
    for entry in dir.flatten() {
        let name = entry.file_name();
        let b = name.as_bytes();
        if b.is_empty() || !b.iter().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let mut len = 6;
        path[..6].copy_from_slice(b"/proc/");
        for &c in b.iter().take(10) {
            if len + 6 >= path.len() {
                break;
            }
            path[len] = c;
            len += 1;
        }
        if len + 5 > path.len() {
            continue;
        }
        path[len..len + 5].copy_from_slice(b"/stat");
        let plen = len + 5;
        let ps = match std::str::from_utf8(&path[..plen]) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let n = read_small(ps, scratch);
        if n == 0 {
            continue;
        }
        let buf = &scratch[..n];

        let open = match buf.iter().position(|&c| c == b'(') {
            Some(p) => p,
            None => continue,
        };
        let close = match buf[open..].iter().position(|&c| c == b')') {
            Some(p) => open + p,
            None => continue,
        };
        let pid: u32 = std::str::from_utf8(&buf[..open])
            .unwrap_or("0")
            .trim()
            .parse()
            .unwrap_or(0);
        if pid == 0 {
            continue;
        }
        let comm = &buf[open + 1..close];
        let rest = &buf[close + 1..];
        let state = rest.iter().find(|&&c| c != b' ').copied().unwrap_or(b'?');
        f(ProcInfo { pid, comm, state });
    }
}

fn cstr_field(p: *const libc::c_char) -> Vec<u8> {
    if p.is_null() {
        return b"unknown".to_vec();
    }
    unsafe {
        let mut n = 0;
        while *p.add(n) != 0 {
            n += 1;
        }
        std::slice::from_raw_parts(p as *const u8, n).to_vec()
    }
}

fn sig_from_name(s: &[u8]) -> Option<i32> {
    const TABLE: &[(&[u8], i32)] = &[
        (b"HUP", 1),
        (b"INT", 2),
        (b"QUIT", 3),
        (b"KILL", 9),
        (b"TERM", 15),
        (b"USR1", 10),
        (b"USR2", 12),
        (b"PIPE", 13),
        (b"ALRM", 14),
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

fn list_signals(out: &mut dyn Write) -> std::io::Result<()> {
    out.write_all(b"HUP INT QUIT ILL TRAP ABRT BUS FPE KILL USR1 SEGV USR2 PIPE ALRM TERM STKFLT CHLD CONT STOP TSTP TTIN TTOU URG XCPU XFSZ VTALRM PROF WINCH IO PWR SYS\n")
}

pub struct PsApplet;
impl Applet for PsApplet {
    fn name(&self) -> &'static str {
        "ps"
    }
    fn description(&self) -> &'static str {
        "Report process status"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        out.write_all(b"  PID STAT COMMAND\n")?;
        let mut scratch = [0u8; 512];
        let mut line: Vec<u8> = Vec::with_capacity(96);
        let mut cmd = [0u8; 256];
        for_each_proc(&mut scratch, |p| {
            line.clear();
            line.extend_from_slice(b"  ");
            push_u64(&mut line, p.pid as u64);
            line.push(b' ');
            line.push(p.state);
            line.push(b' ');

            let mut plen = 6;
            let mut path = [0u8; 32];
            path[..6].copy_from_slice(b"/proc/");
            let mut tmp = [0u8; 12];
            let mut tl = 0;
            let mut v = p.pid;
            if v == 0 {
                tmp[tl] = b'0';
                tl += 1;
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
                if plen + 9 < path.len() {
                    path[plen] = c;
                    plen += 1;
                }
            }
            path[plen..plen + 8].copy_from_slice(b"/cmdline");
            let ps = std::str::from_utf8(&path[..plen + 8]).unwrap_or("");
            let n = read_small(ps, &mut cmd);
            if n > 0 {
                for &c in &cmd[..n] {
                    line.push(if c == 0 { b' ' } else { c });
                }
                while line.last() == Some(&b' ') {
                    line.pop();
                }
            } else {
                line.push(b'[');
                line.extend_from_slice(p.comm);
                line.push(b']');
            }
            line.push(b'\n');
            let _ = out.write_all(&line);
        });
        out.flush()?;
        Ok(0)
    }
}

pub struct KillApplet;
impl Applet for KillApplet {
    fn name(&self) -> &'static str {
        "kill"
    }
    fn description(&self) -> &'static str {
        "Send a signal to processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sig = libc::SIGTERM;
        let mut pids: Vec<i32> = Vec::new();
        let mut i = 0;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-l" {
                list_signals(&mut out)?;
                out.flush()?;
                return Ok(0);
            } else if b == b"-s" {
                i += 1;
                if i >= args.len() {
                    eprintln!("kill: -s needs a signal argument");
                    return Ok(1);
                }
                match sig_from_name(args[i].as_bytes()) {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("kill: unknown signal");
                        return Ok(1);
                    }
                }
            } else if b.len() > 1 && b[0] == b'-' && !b[1].is_ascii_digit() {
                match sig_from_name(&b[1..]) {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("kill: unknown signal");
                        return Ok(1);
                    }
                }
            } else {
                let s = args[i].to_string_lossy();
                match s.parse::<i32>() {
                    Ok(pid) => pids.push(pid),
                    Err(_) => {
                        eprintln!("kill: '{}': invalid pid", s);
                        return Ok(1);
                    }
                }
            }
            i += 1;
        }
        if pids.is_empty() {
            eprintln!("kill: usage: kill [-l] [-SIG] PID...");
            return Ok(1);
        }
        let mut rc = 0;
        for pid in pids {
            if unsafe { libc::kill(pid, sig) } != 0 {
                eprintln!("kill: {}: {}", pid, std::io::Error::last_os_error());
                rc = 1;
            }
        }
        Ok(rc)
    }
}

pub struct KillallApplet;
impl Applet for KillallApplet {
    fn name(&self) -> &'static str {
        "killall"
    }
    fn description(&self) -> &'static str {
        "Send a signal to processes by name"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sig = libc::SIGTERM;
        let mut quiet = false;
        let mut names: Vec<&[u8]> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-q" {
                quiet = true;
            } else if b == b"-l" {
                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                list_signals(&mut out)?;
                return Ok(0);
            } else if b.len() > 1 && b[0] == b'-' && !b[1].is_ascii_digit() {
                match sig_from_name(&b[1..]) {
                    Some(s) => sig = s,
                    None => {
                        eprintln!("killall: unknown signal");
                        return Ok(1);
                    }
                }
            } else {
                names.push(b);
            }
            i += 1;
        }
        if names.is_empty() {
            eprintln!("killall: usage: killall [-q] [-SIG] NAME...");
            return Ok(1);
        }

        let mut scratch = [0u8; 512];
        let mut matched: Vec<u32> = Vec::new();
        for_each_proc(&mut scratch, |p| {
            for want in &names {
                let wbase = match want.iter().rposition(|&c| c == b'/') {
                    Some(pos) => &want[pos + 1..],
                    None => want,
                };
                if p.comm == wbase {
                    matched.push(p.pid);
                    break;
                }
            }
        });
        let mut rc = 0;
        if matched.is_empty() {
            if !quiet {
                eprintln!("killall: no process killed");
            }
            return Ok(1);
        }
        for pid in matched {
            if unsafe { libc::kill(pid as i32, sig) } != 0 {
                eprintln!("killall: {}: {}", pid, std::io::Error::last_os_error());
                rc = 1;
            }
        }
        Ok(rc)
    }
}

pub struct FreeApplet;
impl Applet for FreeApplet {
    fn name(&self) -> &'static str {
        "free"
    }
    fn description(&self) -> &'static str {
        "Display free and used memory"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut div: u64 = 1;
        let mut human = false;
        for a in args {
            let b = a.as_bytes();
            if b == b"-b" || b == b"-k" {
                div = 1;
                human = false;
            } else if b == b"-m" {
                div = 1024;
                human = false;
            } else if b == b"-g" {
                div = 1024 * 1024;
                human = false;
            } else if b == b"-h" {
                human = true;
            }
        }
        let mut buf = [0u8; 4096];
        let n = read_small("/proc/meminfo", &mut buf);
        if n == 0 {
            eprintln!("free: cannot read /proc/meminfo");
            return Ok(1);
        }
        let mem = &buf[..n];
        let total = meminfo_val(mem, b"MemTotal");
        let free = meminfo_val(mem, b"MemFree");
        let shared = meminfo_val(mem, b"Shmem");
        let buffers = meminfo_val(mem, b"Buffers");
        let cached = meminfo_val(mem, b"Cached");
        let stotal = meminfo_val(mem, b"SwapTotal");
        let sfree = meminfo_val(mem, b"SwapFree");
        let used = total.saturating_sub(free);
        let sused = stotal.saturating_sub(sfree);

        fn scaled(v: u64, div: u64, human: bool, out: &mut Vec<u8>) {
            if human {
                let bytes = v.saturating_mul(1024);
                let (q, suf) = if bytes >= 1 << 30 {
                    (bytes / (1 << 30), "G")
                } else if bytes >= 1 << 20 {
                    (bytes / (1 << 20), "M")
                } else {
                    (bytes / (1 << 10), "K")
                };
                push_u64(out, q);
                out.extend_from_slice(suf.as_bytes());
            } else if div == 1 {
                push_u64(out, v);
            } else {
                push_u64(out, v.div_ceil(div));
            }
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line: Vec<u8> = Vec::with_capacity(128);
        out.write_all(
            b"              total        used        free      shared     buffers      cached\n",
        )?;
        line.extend_from_slice(b"Mem:");
        for v in [total, used, free, shared, buffers, cached] {
            line.push(b' ');
            let mut cell: Vec<u8> = Vec::with_capacity(12);
            scaled(v, div, human, &mut cell);

            while cell.len() < 11 {
                line.push(b' ');
            }
            line.extend_from_slice(&cell);
        }
        line.push(b'\n');
        out.write_all(&line)?;
        line.clear();
        line.extend_from_slice(b"Swap:");
        let mut cell: Vec<u8> = Vec::with_capacity(12);
        for v in [stotal, sused, sfree] {
            cell.clear();
            scaled(v, div, human, &mut cell);
            while cell.len() < 11 {
                line.push(b' ');
            }
            line.extend_from_slice(&cell);
            line.push(b' ');
        }

        while line.last() == Some(&b' ') {
            line.pop();
        }
        line.push(b'\n');
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}

pub struct UptimeApplet;
impl Applet for UptimeApplet {
    fn name(&self) -> &'static str {
        "uptime"
    }
    fn description(&self) -> &'static str {
        "Show how long the system has been running"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let since = args.iter().any(|a| a.as_bytes() == b"-s");
        let mut ub = [0u8; 64];
        let n = read_small("/proc/uptime", &mut ub);
        if n == 0 {
            eprintln!("uptime: cannot read /proc/uptime");
            return Ok(1);
        }

        let mut secs: u64 = 0;
        let mut frac: u64 = 0;
        let mut i = 0;
        while i < n && ub[i].is_ascii_digit() {
            secs = secs
                .saturating_mul(10)
                .saturating_add((ub[i] - b'0') as u64);
            i += 1;
        }
        if i < n && ub[i] == b'.' {
            i += 1;
            let mut mul = 100;
            while i < n && ub[i].is_ascii_digit() && mul > 0 {
                frac += ((ub[i] - b'0') as u64) * mul;
                mul /= 10;
                i += 1;
            }
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line: Vec<u8> = Vec::with_capacity(128);
        if since {
            let now = unsafe { libc::time(std::ptr::null_mut()) } as i64 - secs as i64;
            let mut tm: libc::tm = unsafe { std::mem::zeroed() };
            unsafe { libc::localtime_r(&now, &mut tm) };
            fn p2(out: &mut Vec<u8>, v: i32) {
                out.push(b'0' + (v / 10) as u8);
                out.push(b'0' + (v % 10) as u8);
            }
            push_u64(&mut line, (tm.tm_year + 1900) as u64);
            line.push(b'-');
            p2(&mut line, tm.tm_mon + 1);
            line.push(b'-');
            p2(&mut line, tm.tm_mday);
            line.push(b' ');
            p2(&mut line, tm.tm_hour);
            line.push(b':');
            p2(&mut line, tm.tm_min);
            line.push(b':');
            p2(&mut line, tm.tm_sec);
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            return Ok(0);
        }

        let now = unsafe { libc::time(std::ptr::null_mut()) };
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        unsafe { libc::localtime_r(&now, &mut tm) };
        fn p2b(out: &mut Vec<u8>, v: i32) {
            out.push(b'0' + ((v / 10) % 10) as u8);
            out.push(b'0' + (v % 10) as u8);
        }
        line.push(b' ');
        p2b(&mut line, tm.tm_hour);
        line.push(b':');
        p2b(&mut line, tm.tm_min);
        line.push(b':');
        p2b(&mut line, tm.tm_sec);
        line.extend_from_slice(b" up ");
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        if days > 0 {
            push_u64(&mut line, days);
            line.extend_from_slice(if days == 1 { b" day, " } else { b" days, " });
        }
        if days > 0 || hours > 0 {
            push_u64(&mut line, hours);
            line.push(b':');
            if mins < 10 {
                line.push(b'0');
            }
            push_u64(&mut line, mins);
        } else {
            push_u64(&mut line, mins);
            line.extend_from_slice(b" min");
        }

        let mut lb = [0u8; 64];
        let ln = read_small("/proc/loadavg", &mut lb);
        if ln > 0 {
            line.extend_from_slice(b",  load average: ");
            let mut fields = 0;
            for &c in &lb[..ln] {
                if c == b'\n' {
                    break;
                }
                line.push(c);
                if c == b' ' {
                    fields += 1;
                    if fields == 3 {
                        line.pop();
                        break;
                    }
                }
            }
        }
        let _ = frac;
        line.push(b'\n');
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}

pub struct UnameApplet;
impl Applet for UnameApplet {
    fn name(&self) -> &'static str {
        "uname"
    }
    fn description(&self) -> &'static str {
        "Print system information"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mask: u8 = 0;
        for a in args {
            let b = a.as_bytes();
            if b.first() == Some(&b'-') && b.len() > 1 && b[1] != b'-' {
                for &c in &b[1..] {
                    match c {
                        b's' => mask |= 1,
                        b'n' => mask |= 2,
                        b'r' => mask |= 4,
                        b'v' => mask |= 8,
                        b'm' => mask |= 16,
                        b'p' => mask |= 32,
                        b'i' => mask |= 64,
                        b'o' => mask |= 128,
                        b'a' => {
                            mask = 0xFF;
                            break;
                        }
                        _ => {}
                    }
                }
            } else if b == b"--all" {
                mask = 0xFF;
            }
        }
        if mask == 0 {
            mask = 1;
        }
        let mut uts: libc::utsname = unsafe { std::mem::zeroed() };
        if unsafe { libc::uname(&mut uts) } != 0 {
            eprintln!("uname: cannot get system name");
            return Ok(1);
        }
        let fields: [Vec<u8>; 8] = [
            cstr_field(uts.sysname.as_ptr()),
            cstr_field(uts.nodename.as_ptr()),
            cstr_field(uts.release.as_ptr()),
            cstr_field(uts.version.as_ptr()),
            cstr_field(uts.machine.as_ptr()),
            cstr_field(uts.machine.as_ptr()),
            cstr_field(uts.machine.as_ptr()),
            b"GNU/Linux".to_vec(),
        ];
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut first = true;

        for (i, f) in fields.iter().enumerate() {
            if mask & (1 << i) != 0 {
                if !first {
                    out.write_all(b" ")?;
                }
                first = false;
                if mask == 0xFF && f == b"unknown" {
                    continue;
                }
                out.write_all(f)?;
            }
        }
        out.write_all(b"\n")?;
        out.flush()?;
        Ok(0)
    }
}

pub struct HostnameApplet;
impl Applet for HostnameApplet {
    fn name(&self) -> &'static str {
        "hostname"
    }
    fn description(&self) -> &'static str {
        "Get or set the hostname"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            let mut buf = [0u8; 65];
            if unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, 64) } != 0 {
                eprintln!("hostname: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            buf[64] = 0;
            let len = buf.iter().position(|&c| c == 0).unwrap_or(64);
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            out.write_all(&buf[..len])?;
            out.write_all(b"\n")?;
            out.flush()?;
            Ok(0)
        } else {
            let b = args[0].as_bytes();
            if unsafe { libc::sethostname(b.as_ptr() as *const libc::c_char, b.len()) } != 0 {
                eprintln!("hostname: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            Ok(0)
        }
    }
}

fn pw_name(uid: libc::uid_t) -> Option<Vec<u8>> {
    unsafe {
        let pw = libc::getpwuid(uid);
        if pw.is_null() || (*pw).pw_name.is_null() {
            return None;
        }
        Some(cstr_field((*pw).pw_name))
    }
}

fn gr_name(gid: libc::gid_t) -> Option<Vec<u8>> {
    unsafe {
        let gr = libc::getgrgid(gid);
        if gr.is_null() || (*gr).gr_name.is_null() {
            return None;
        }
        Some(cstr_field((*gr).gr_name))
    }
}

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

pub struct LognameApplet;
impl Applet for LognameApplet {
    fn name(&self) -> &'static str {
        "logname"
    }
    fn description(&self) -> &'static str {
        "Print the name of the current user"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if !args.is_empty() {
            eprintln!("logname: too many arguments");
            return Ok(1);
        }
        unsafe {
            let p = libc::getlogin();
            if !p.is_null() {
                let name = cstr_field(p);
                if !name.is_empty() {
                    let stdout = std::io::stdout();
                    let mut out = stdout.lock();
                    out.write_all(&name)?;
                    out.write_all(b"\n")?;
                    out.flush()?;
                    return Ok(0);
                }
            }
        }

        if let Some(user) = crate::core::platform::get_current_username() {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            out.write_all(user.as_bytes())?;
            out.write_all(b"\n")?;
            out.flush()?;
            Ok(0)
        } else {
            eprintln!("logname: no login name");
            Ok(1)
        }
    }
}
