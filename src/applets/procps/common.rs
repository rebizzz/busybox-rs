use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;

unsafe extern "C" {
    pub fn gethostbyname(name: *const libc::c_char) -> *mut libc::hostent;
}


pub fn push_u64(out: &mut Vec<u8>, mut v: u64) {
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

pub fn read_small(path: &str, buf: &mut [u8]) -> usize {
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

pub fn meminfo_val(buf: &[u8], key: &[u8]) -> u64 {
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

pub struct ProcInfo<'a> {
    pub pid: u32,
    pub comm: &'a [u8],
    pub state: u8,
}

pub fn for_each_proc<F: FnMut(ProcInfo<'_>)>(scratch: &mut [u8; 512], mut f: F) {
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

pub fn cstr_field(p: *const libc::c_char) -> Vec<u8> {
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

pub fn sig_from_name(s: &[u8]) -> Option<i32> {
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

pub fn list_signals(out: &mut dyn Write) -> std::io::Result<()> {
    out.write_all(b"HUP INT QUIT ILL TRAP ABRT BUS FPE KILL USR1 SEGV USR2 PIPE ALRM TERM STKFLT CHLD CONT STOP TSTP TTIN TTOU URG XCPU XFSZ VTALRM PROF WINCH IO PWR SYS\n")
}





pub fn proc_path(pid: u32, tail: &[u8], out: &mut [u8; 48]) -> usize {
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

pub fn proc_pids() -> Vec<u32> {
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

pub struct PStat {
    pub pid: u32,
    pub comm: Vec<u8>,
    pub state: u8,
    pub utime: u64,
    pub stime: u64,
    pub starttime: u64,
    pub vsize: u64,
    pub rss_pages: i64,
}

pub fn parse_num(b: &[u8]) -> u64 {
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

pub fn read_stat(pid: u32) -> Option<PStat> {
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

pub fn read_cmdline(pid: u32, buf: &mut [u8]) -> usize {
    let mut path = [0u8; 48];
    let n = proc_path(pid, b"/cmdline", &mut path);
    match std::str::from_utf8(&path[..n]) {
        Ok(ps) => read_small(ps, buf),
        Err(_) => 0,
    }
}

pub fn read_uids(pid: u32) -> Option<(u32, u32)> {
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

pub fn uid_of_name(name: &[u8]) -> Option<u32> {
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

pub fn user_name(uid: u32) -> Vec<u8> {
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

pub fn strerror_last() -> String {
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


pub fn exec_prog(prog: &[u8], args: &[OsString]) -> i32 {
    exec_prog_argv0(prog, prog, args)
}

pub fn exec_prog_argv0(prog: &[u8], argv0: &[u8], args: &[OsString]) -> i32 {
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

pub struct PgrepOpts<'a> {
    pub full: bool,
    pub exact: bool,
    pub newest: bool,
    pub oldest: bool,
    pub invert: bool,
    pub count: bool,
    pub list: bool,
    pub uid: Option<u32>,
    pub euid: Option<u32>,
    pub signal: i32,
    pub pattern: Option<&'a [u8]>,
}

pub fn parse_pgrep(
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

pub fn pgrep_match(st: &PStat, cmd: &[u8], o: &PgrepOpts<'_>) -> bool {
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

pub fn collect_matches(o: &PgrepOpts<'_>) -> Vec<(u32, Vec<u8>, u64)> {
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



pub fn read_cpu_total() -> u64 {
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

pub fn parse_delay(b: &[u8]) -> u64 {
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



pub fn meminfo_kb(key: &[u8]) -> u64 {
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

pub fn vmstat_val(key: &[u8]) -> u64 {
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

pub fn trailing_num(ln: &[u8]) -> u64 {
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

pub fn vmstat_line(out: &mut dyn Write) -> std::io::Result<()> {
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



pub fn pw_name(uid: libc::uid_t) -> Option<Vec<u8>> {
    unsafe {
        let pw = libc::getpwuid(uid);
        if pw.is_null() || (*pw).pw_name.is_null() {
            return None;
        }
        Some(cstr_field((*pw).pw_name))
    }
}

pub fn gr_name(gid: libc::gid_t) -> Option<Vec<u8>> {
    unsafe {
        let gr = libc::getgrgid(gid);
        if gr.is_null() || (*gr).gr_name.is_null() {
            return None;
        }
        Some(cstr_field((*gr).gr_name))
    }
}
