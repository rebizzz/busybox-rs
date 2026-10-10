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

fn exec_prog(prog: &[u8], args: &[OsString]) -> i32 {
    use std::ffi::CString;
    let argv0 = CString::new(prog).unwrap_or_else(|_| CString::new("sh").unwrap());
    let mut cs: Vec<CString> = Vec::with_capacity(args.len() + 1);
    cs.push(argv0);
    for a in args {
        cs.push(CString::new(a.as_bytes()).unwrap_or_else(|_| CString::new("").unwrap()));
    }
    let mut ptrs: Vec<*const libc::c_char> = cs.iter().map(|c| c.as_ptr()).collect();
    ptrs.push(std::ptr::null());
    unsafe {
        libc::execvp(ptrs[0], ptrs.as_ptr());
    }
    eprintln!(
        "{}: {}",
        String::from_utf8_lossy(prog),
        std::io::Error::last_os_error()
    );
    127
}

fn parse_u64_suffix(b: &[u8]) -> Option<u64> {
    if b.is_empty() {
        return None;
    }
    let (num, mul) = match b.last() {
        Some(c) if c.is_ascii_alphabetic() => (
            &b[..b.len() - 1],
            match c.to_ascii_uppercase() {
                b'K' => 1024u64,
                b'M' => 1024 * 1024,
                b'G' => 1024 * 1024 * 1024,
                b'T' => 1024 * 1024 * 1024 * 1024,
                _ => return None,
            },
        ),
        _ => (b, 1u64),
    };
    if num.is_empty() || !num.iter().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut v: u64 = 0;
    for &c in num {
        v = v.checked_mul(10)?.checked_add((c - b'0') as u64)?;
    }
    v.checked_mul(mul)
}

const PER_LINUX: libc::c_ulong = 0x0000;
const PER_LINUX32: libc::c_ulong = 0x0008;

fn do_personality(persona: libc::c_ulong, prog: Option<Vec<OsString>>) -> i32 {
    let cur = unsafe { libc::personality(0xffffffff) };
    if cur < 0 {
        eprintln!("personality: {}", std::io::Error::last_os_error());
        return 1;
    }
    if unsafe { libc::personality(persona) } < 0 {
        eprintln!("personality: {}", std::io::Error::last_os_error());
        return 1;
    }
    match prog {
        Some(cmd) if !cmd.is_empty() => exec_prog(cmd[0].as_bytes(), &cmd[1..]),
        _ => {
            let sh = b"/bin/sh";
            exec_prog(sh, &[])
        }
    }
}

pub struct Linux32Applet;
impl Applet for Linux32Applet {
    fn name(&self) -> &'static str {
        "linux32"
    }
    fn description(&self) -> &'static str {
        "Run a program with 32-bit personality"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut cmd: Vec<OsString> = Vec::new();
        for a in args {
            if cmd.is_empty() && a.as_bytes() == b"--32" {
                continue;
            } else if cmd.is_empty() && a.as_bytes().first() == Some(&b'-') {
                eprintln!("linux32: unknown option");
                return Ok(1);
            } else {
                cmd.push(a.clone());
            }
        }
        let prog = if cmd.is_empty() { None } else { Some(cmd) };
        Ok(do_personality(PER_LINUX32, prog))
    }
}

pub struct Linux64Applet;
impl Applet for Linux64Applet {
    fn name(&self) -> &'static str {
        "linux64"
    }
    fn description(&self) -> &'static str {
        "Run a program with 64-bit personality"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut cmd: Vec<OsString> = Vec::new();
        for a in args {
            if cmd.is_empty() && a.as_bytes() == b"--64" {
                continue;
            } else if cmd.is_empty() && a.as_bytes().first() == Some(&b'-') {
                eprintln!("linux64: unknown option");
                return Ok(1);
            } else {
                cmd.push(a.clone());
            }
        }
        let prog = if cmd.is_empty() { None } else { Some(cmd) };
        Ok(do_personality(PER_LINUX, prog))
    }
}

pub struct SetarchApplet;
impl Applet for SetarchApplet {
    fn name(&self) -> &'static str {
        "setarch"
    }
    fn description(&self) -> &'static str {
        "Change reported architecture and run a program"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut arch: Option<&[u8]> = None;
        let mut at: Option<usize> = None;
        let mut no_randomize = false;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--32" {
                arch = Some(b"linux32");
            } else if b == b"--64" {
                arch = Some(b"linux64");
            } else if b == b"-R" {
                no_randomize = true;
            } else if b.first() == Some(&b'-') {
                eprintln!("setarch: unknown option");
                return Ok(1);
            } else if arch.is_none() {
                arch = Some(b);
            } else {
                at = Some(i);
                break;
            }
            i += 1;
        }
        let arch = match arch {
            Some(a) => a,
            None => {
                eprintln!("usage: setarch ARCH [PROG...]");
                return Ok(1);
            }
        };

        let mut persona: libc::c_ulong = if arch.eq_ignore_ascii_case(b"i386")
            || arch.eq_ignore_ascii_case(b"i486")
            || arch.eq_ignore_ascii_case(b"i586")
            || arch.eq_ignore_ascii_case(b"i686")
            || arch.eq_ignore_ascii_case(b"linux32")
        {
            PER_LINUX32
        } else if arch.eq_ignore_ascii_case(b"x86_64")
            || arch.eq_ignore_ascii_case(b"amd64")
            || arch.eq_ignore_ascii_case(b"linux64")
        {
            PER_LINUX
        } else {
            eprintln!(
                "setarch: unknown architecture '{}'",
                String::from_utf8_lossy(arch)
            );
            return Ok(1);
        };
        if no_randomize {
            persona |= 0x0040000;
        }
        let prog = at.map(|p| args[p..].to_vec());
        Ok(do_personality(persona, prog))
    }
}

pub struct PivotRootApplet;
impl Applet for PivotRootApplet {
    fn name(&self) -> &'static str {
        "pivot_root"
    }
    fn description(&self) -> &'static str {
        "Change the root filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() != 2 {
            eprintln!("usage: pivot_root NEWROOT PUT_OLD");
            return Ok(1);
        }
        use std::ffi::CString;
        let new = CString::new(args[0].as_bytes()).unwrap_or_else(|_| CString::new("/").unwrap());
        let old = CString::new(args[1].as_bytes()).unwrap_or_else(|_| CString::new("/").unwrap());

        let r = unsafe {
            libc::syscall(
                libc::SYS_pivot_root as libc::c_long,
                new.as_ptr(),
                old.as_ptr(),
            )
        };
        if r != 0 {
            eprintln!("pivot_root: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}

fn chdir_cstr(p: &[u8]) -> bool {
    use std::ffi::CString;
    match CString::new(p) {
        Ok(c) => unsafe { libc::chdir(c.as_ptr()) == 0 },
        Err(_) => false,
    }
}

pub struct SwitchRootApplet;
impl Applet for SwitchRootApplet {
    fn name(&self) -> &'static str {
        "switch_root"
    }
    fn description(&self) -> &'static str {
        "Switch to another filesystem as the root"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut console: Option<Vec<u8>> = None;
        let mut pos: Vec<OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-c" {
                i += 1;
                if i >= args.len() {
                    eprintln!("switch_root: -c needs a console");
                    return Ok(1);
                }
                console = Some(args[i].as_bytes().to_vec());
            } else if b.first() == Some(&b'-') {
                eprintln!("switch_root: unknown option");
                return Ok(1);
            } else {
                pos.push(args[i].clone());
            }
            i += 1;
        }
        if pos.len() < 2 {
            eprintln!("usage: switch_root [-c CONSOLE] NEWROOT INIT [ARGS...]");
            return Ok(1);
        }
        use std::ffi::CString;
        let newroot = pos[0].as_bytes().to_vec();

        if !chdir_cstr(&newroot) {
            eprintln!(
                "switch_root: '{}': {}",
                pos[0].to_string_lossy(),
                std::io::Error::last_os_error()
            );
            return Ok(1);
        }
        let dot = CString::new(".").unwrap();
        let slash = CString::new("/").unwrap();
        unsafe {
            if libc::mount(
                dot.as_ptr(),
                slash.as_ptr(),
                std::ptr::null(),
                libc::MS_MOVE,
                std::ptr::null(),
            ) != 0
            {
                eprintln!(
                    "switch_root: mount --move: {}",
                    std::io::Error::last_os_error()
                );
                return Ok(1);
            }
            if libc::chroot(dot.as_ptr()) != 0 {
                eprintln!("switch_root: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            if libc::chdir(slash.as_ptr()) != 0 {
                eprintln!("switch_root: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
        }
        if let Some(c) = console {
            let _ = c;
        }
        let cmd: Vec<OsString> = pos[1..].to_vec();
        Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
    }
}

pub struct RunInitApplet;
impl Applet for RunInitApplet {
    fn name(&self) -> &'static str {
        "run-init"
    }
    fn description(&self) -> &'static str {
        "Run init from a new root filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut console: Option<Vec<u8>> = None;
        let mut pos: Vec<OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-d" {
                i += 1;
                if i >= args.len() {
                    eprintln!("run-init: -d needs a console");
                    return Ok(1);
                }
                console = Some(args[i].as_bytes().to_vec());
            } else if b.first() == Some(&b'-') {
                eprintln!("run-init: unknown option");
                return Ok(1);
            } else {
                pos.push(args[i].clone());
            }
            i += 1;
        }
        if pos.len() < 2 {
            eprintln!("usage: run-init [-d CONSOLE] NEWROOT INIT [ARGS...]");
            return Ok(1);
        }
        use std::ffi::CString;
        if !chdir_cstr(pos[0].as_bytes()) {
            eprintln!(
                "run-init: '{}': {}",
                pos[0].to_string_lossy(),
                std::io::Error::last_os_error()
            );
            return Ok(1);
        }
        let dot = CString::new(".").unwrap();
        let slash = CString::new("/").unwrap();
        unsafe {
            if libc::mount(
                dot.as_ptr(),
                slash.as_ptr(),
                std::ptr::null(),
                libc::MS_MOVE,
                std::ptr::null(),
            ) != 0
            {
                eprintln!(
                    "run-init: mount --move: {}",
                    std::io::Error::last_os_error()
                );
                return Ok(1);
            }
            if libc::chroot(dot.as_ptr()) != 0 {
                eprintln!("run-init: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
        }
        if let Some(c) = console {
            use std::ffi::CString as CS;
            if let Ok(cp) = CS::new(c) {
                unsafe {
                    let fd = libc::open(cp.as_ptr(), libc::O_RDWR);
                    if fd >= 0 {
                        libc::dup2(fd, 0);
                        libc::dup2(fd, 1);
                        libc::dup2(fd, 2);
                        if fd > 2 {
                            libc::close(fd);
                        }
                    }
                }
            }
        }
        let cmd: Vec<OsString> = pos[1..].to_vec();
        Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
    }
}

pub struct AdjtimexApplet;
impl Applet for AdjtimexApplet {
    fn name(&self) -> &'static str {
        "adjtimex"
    }
    fn description(&self) -> &'static str {
        "Print or set kernel time variables"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut tx: libc::timex = unsafe { std::mem::zeroed() };
        tx.modes = 0;
        let r = unsafe { libc::adjtimex(&mut tx) };
        if r < 0 {
            eprintln!("adjtimex: {}", std::io::Error::last_os_error());
            return Ok(1);
        }

        let mut set_modes: u32 = 0;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            let next = || -> Option<i64> {
                if i + 1 >= args.len() {
                    return None;
                }
                std::str::from_utf8(args[i + 1].as_bytes())
                    .ok()?
                    .parse::<i64>()
                    .ok()
            };
            if b == b"--print" || b == b"-p" {
            } else if b == b"-o" {
                match next() {
                    Some(v) => {
                        tx.offset = v as libc::c_long;
                        set_modes |= 1;
                    }
                    None => {
                        eprintln!("adjtimex: -o needs a value");
                        return Ok(1);
                    }
                }
                i += 1;
            } else if b == b"-f" {
                match next() {
                    Some(v) => {
                        tx.freq = v as libc::c_long;
                        set_modes |= 2;
                    }
                    None => {
                        eprintln!("adjtimex: -f needs a value");
                        return Ok(1);
                    }
                }
                i += 1;
            } else if b == b"-t" {
                match next() {
                    Some(v) => {
                        tx.tick = v as libc::c_long;
                        set_modes |= 4;
                    }
                    None => {
                        eprintln!("adjtimex: -t needs a value");
                        return Ok(1);
                    }
                }
                i += 1;
            } else {
                eprintln!("adjtimex: unknown option");
                return Ok(1);
            }
            i += 1;
        }
        if set_modes != 0 {
            tx.modes = set_modes;
            if unsafe { libc::adjtimex(&mut tx) } < 0 {
                eprintln!("adjtimex: {}", std::io::Error::last_os_error());
                return Ok(1);
            }

            tx.modes = 0;
            if unsafe { libc::adjtimex(&mut tx) } < 0 {
                eprintln!("adjtimex: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line = Vec::with_capacity(160);
        for (k, v) in [
            ("mode", tx.modes as i64),
            ("offset", tx.offset as i64),
            ("freq", tx.freq as i64),
            ("tick", tx.tick as i64),
            ("status", tx.status as i64),
        ] {
            line.extend_from_slice(k.as_bytes());
            line.extend_from_slice(b": ");
            if v < 0 {
                line.push(b'-');
                push_u64(&mut line, v.unsigned_abs());
            } else {
                push_u64(&mut line, v as u64);
            }
            line.push(b'\n');
        }
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}

const RTC_RD_TIME: libc::c_ulong = 0x8024_7009;

pub struct HwclockApplet;
impl Applet for HwclockApplet {
    fn name(&self) -> &'static str {
        "hwclock"
    }
    fn description(&self) -> &'static str {
        "Read or set the hardware clock"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut utc = false;
        let mut hctosys = false;
        let mut systohc = false;
        for a in args {
            let b = a.as_bytes();
            if b == b"-r" || b == b"--show" {
            } else if b == b"-u" || b == b"--utc" {
                utc = true;
            } else if b == b"-l" || b == b"--localtime" {
                utc = false;
            } else if b == b"--hctosys" || b == b"-s" {
                hctosys = true;
            } else if b == b"--systohc" || b == b"-w" {
                systohc = true;
            } else {
                eprintln!("hwclock: unknown option");
                return Ok(1);
            }
        }
        if hctosys || systohc {
            eprintln!("hwclock: cannot access hardware clock: no RTC device");
            return Ok(1);
        }

        let mut rtc: [i32; 9] = [0; 9];
        let mut got_rtc = false;
        for dev in ["/dev/rtc0", "/dev/rtc"] {
            use std::ffi::CString;
            if let Ok(c) = CString::new(dev) {
                let fd = unsafe { libc::open(c.as_ptr(), libc::O_RDONLY) };
                if fd >= 0 {
                    if unsafe { libc::ioctl(fd, RTC_RD_TIME, rtc.as_mut_ptr()) } == 0 {
                        got_rtc = true;
                    }
                    unsafe { libc::close(fd) };
                    if got_rtc {
                        break;
                    }
                }
            }
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        if got_rtc {
            let mut line = Vec::with_capacity(48);
            push_u64(&mut line, (rtc[5] + 1900) as u64);
            line.push(b'-');
            let m = (rtc[4] + 1) as u8;
            line.push(b'0' + m / 10);
            line.push(b'0' + m % 10);
            line.push(b'-');
            line.push(b'0' + (rtc[3] / 10) as u8);
            line.push(b'0' + (rtc[3] % 10) as u8);
            line.push(b' ');
            for v in [rtc[2], rtc[1], rtc[0]] {
                line.push(b'0' + (v / 10) as u8);
                line.push(b'0' + (v % 10) as u8);
                line.push(b':');
            }
            line.pop();
            if utc {
                line.extend_from_slice(b" UTC");
            }
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            return Ok(0);
        }

        let now = unsafe { libc::time(std::ptr::null_mut()) };
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        if utc {
            unsafe { libc::gmtime_r(&now, &mut tm) };
        } else {
            unsafe { libc::localtime_r(&now, &mut tm) };
        }
        eprintln!("hwclock: no RTC device, showing system time");
        let mut line = Vec::with_capacity(48);
        push_u64(&mut line, (tm.tm_year + 1900) as u64);
        line.push(b'-');
        line.push(b'0' + ((tm.tm_mon + 1) / 10) as u8);
        line.push(b'0' + ((tm.tm_mon + 1) % 10) as u8);
        line.push(b'-');
        line.push(b'0' + (tm.tm_mday / 10) as u8);
        line.push(b'0' + (tm.tm_mday % 10) as u8);
        line.push(b' ');
        for v in [tm.tm_hour, tm.tm_min, tm.tm_sec] {
            line.push(b'0' + (v / 10) as u8);
            line.push(b'0' + (v % 10) as u8);
            line.push(b':');
        }
        line.pop();
        line.push(b'\n');
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}

pub struct RtcwakeApplet;
impl Applet for RtcwakeApplet {
    fn name(&self) -> &'static str {
        "rtcwake"
    }
    fn description(&self) -> &'static str {
        "Set RTC wakeup alarm and suspend"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mode = [0u8; 16];
        let mut mode_len = 0usize;
        let mut seconds: u64 = 0;
        let mut have_mode = false;
        let mut have_time = false;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-m" || b.starts_with(b"--mode") {
                let v: &[u8];
                if b == b"-m" {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("rtcwake: -m needs a mode");
                        return Ok(1);
                    }
                    v = args[i].as_bytes();
                } else if let Some(eq) = b.iter().position(|&c| c == b'=') {
                    v = &b[eq + 1..];
                } else {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("rtcwake: --mode needs a value");
                        return Ok(1);
                    }
                    v = args[i].as_bytes();
                }
                let n = v.len().min(mode.len());
                mode[..n].copy_from_slice(&v[..n]);
                mode_len = n;
                have_mode = true;
            } else if b == b"-s" || b.starts_with(b"--seconds") {
                let v: &[u8];
                if b == b"-s" {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("rtcwake: -s needs seconds");
                        return Ok(1);
                    }
                    v = args[i].as_bytes();
                } else if let Some(eq) = b.iter().position(|&c| c == b'=') {
                    v = &b[eq + 1..];
                } else {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("rtcwake: --seconds needs a value");
                        return Ok(1);
                    }
                    v = args[i].as_bytes();
                }
                match std::str::from_utf8(v)
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                {
                    Some(s) => {
                        seconds = s;
                        have_time = true;
                    }
                    None => {
                        eprintln!("rtcwake: invalid seconds");
                        return Ok(1);
                    }
                }
            } else {
                eprintln!("rtcwake: unknown option");
                return Ok(1);
            }
            i += 1;
        }
        if !have_mode || !have_time {
            eprintln!("usage: rtcwake -m MODE -s SECONDS");
            return Ok(1);
        }

        let mut rtc: [i32; 9] = [0; 9];
        let mut fd = -1;
        for dev in ["/dev/rtc0", "/dev/rtc"] {
            use std::ffi::CString;
            if let Ok(c) = CString::new(dev) {
                let f = unsafe { libc::open(c.as_ptr(), libc::O_RDWR) };
                if f >= 0 {
                    fd = f;
                    break;
                }
            }
        }
        if fd < 0 {
            eprintln!("rtcwake: cannot open RTC device: no such device");
            return Ok(1);
        }
        let ok = unsafe { libc::ioctl(fd, RTC_RD_TIME, rtc.as_mut_ptr()) } == 0;
        unsafe { libc::close(fd) };
        if !ok {
            eprintln!("rtcwake: cannot read RTC time");
            return Ok(1);
        }

        let now = unsafe { libc::time(std::ptr::null_mut()) } as u64;
        let wake = now.saturating_add(seconds);
        let mut s = Vec::with_capacity(24);
        push_u64(&mut s, wake);
        match std::fs::write("/sys/class/rtc/rtc0/wakealarm", &s) {
            Ok(()) => {
                eprintln!(
                    "rtcwake: wakeup set {}s from now (mode {})",
                    seconds,
                    String::from_utf8_lossy(&mode[..mode_len])
                );
                Ok(0)
            }
            Err(e) => {
                eprintln!("rtcwake: cannot set wakealarm: {}", e);
                Ok(1)
            }
        }
    }
}

pub struct SeedrngApplet;
impl Applet for SeedrngApplet {
    fn name(&self) -> &'static str {
        "seedrng"
    }
    fn description(&self) -> &'static str {
        "Seed the kernel random number generator"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.iter().any(|a| a.as_bytes().first() == Some(&b'-')) {
            eprintln!("seedrng: no options supported in this subset");
            return Ok(1);
        }

        let mut seed = [0u8; 512];
        let mut got = 0;
        while got < seed.len() {
            let r = unsafe {
                libc::getrandom(
                    seed[got..].as_mut_ptr() as *mut libc::c_void,
                    seed.len() - got,
                    0,
                )
            };
            if r < 0 {
                let e = std::io::Error::last_os_error();
                if e.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                eprintln!("seedrng: getrandom: {}", e);
                return Ok(1);
            }
            got += r as usize;
        }

        const RNDADDENTROPY: libc::c_ulong = 0x4008_5203;
        use std::ffi::CString;
        let mut rc = 0;
        if let Ok(dev) = CString::new("/dev/urandom") {
            let fd = unsafe { libc::open(dev.as_ptr(), libc::O_WRONLY) };
            if fd < 0 {
                eprintln!(
                    "seedrng: cannot open /dev/urandom: {}",
                    std::io::Error::last_os_error()
                );
                return Ok(1);
            }

            let mut req = Vec::with_capacity(8 + 512);
            req.extend_from_slice(&256u32.to_ne_bytes());
            req.extend_from_slice(&512u32.to_ne_bytes());
            req.extend_from_slice(&seed);
            if unsafe { libc::ioctl(fd, RNDADDENTROPY, req.as_ptr()) } != 0 {
                eprintln!(
                    "seedrng: RNDADDENTROPY: {}",
                    std::io::Error::last_os_error()
                );
                rc = 1;
            }
            unsafe { libc::close(fd) };
        }
        Ok(rc)
    }
}

pub struct SmemcapApplet;
impl Applet for SmemcapApplet {
    fn name(&self) -> &'static str {
        "smemcap"
    }
    fn description(&self) -> &'static str {
        "Report process memory from smaps (PSS)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() > 1 {
            eprintln!("usage: smemcap [FILE]");
            return Ok(1);
        }

        let mut rows: Vec<(u32, Vec<u8>, u64, u64)> = Vec::new();
        let dir = match std::fs::read_dir("/proc") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("smemcap: {}", e);
                return Ok(1);
            }
        };
        for e in dir.flatten() {
            let b = e.file_name();
            let bb = b.as_bytes();
            if bb.is_empty() || !bb.iter().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let pid: u32 = std::str::from_utf8(bb)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let mut path = [0u8; 64];
            let base = b"/proc/";
            path[..6].copy_from_slice(base);
            let mut len = 6;
            for &c in bb.iter().take(10) {
                if len + 7 < path.len() {
                    path[len] = c;
                    len += 1;
                }
            }
            path[len..len + 6].copy_from_slice(b"/smaps");
            let plen = len + 6;
            let ps = match std::str::from_utf8(&path[..plen]) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut sb = [0u8; 65536];
            let rn = read_small(ps, &mut sb);
            if rn == 0 {
                continue;
            }
            let mut rss: u64 = 0;
            let mut pss: u64 = 0;
            let mut k = 0;
            while k < rn {
                let mut en = k;
                while en < rn && sb[en] != b'\n' {
                    en += 1;
                }
                let l = &sb[k..en];
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
                k = en + 1;
            }

            let mut comm = Vec::new();
            let mut stp = [0u8; 64];
            stp[..6].copy_from_slice(base);
            let mut sl = 6;
            for &c in bb.iter().take(10) {
                if sl + 5 < stp.len() {
                    stp[sl] = c;
                    sl += 1;
                }
            }
            stp[sl..sl + 5].copy_from_slice(b"/stat");
            if let Ok(ss) = std::str::from_utf8(&stp[..sl + 5]) {
                let mut buf = [0u8; 512];
                let n = read_small(ss, &mut buf);
                if let Some(o) = buf[..n].iter().position(|&c| c == b'(') {
                    if let Some(cl) = buf[o..n].iter().position(|&c| c == b')') {
                        comm = buf[o + 1..o + cl].to_vec();
                    }
                }
            }
            rows.push((pid, comm, pss, rss));
        }
        rows.sort_by_key(|t| t.0);
        let mut out = Vec::with_capacity(4096);
        out.extend_from_slice(b"PID COMMAND PSS(kB) RSS(kB)\n");
        let mut tp = 0u64;
        let mut tr = 0u64;
        for (pid, comm, pss, rss) in &rows {
            tp += *pss;
            tr += *rss;
            push_u64(&mut out, *pid as u64);
            out.push(b' ');
            out.extend_from_slice(comm);
            out.push(b' ');
            push_u64(&mut out, *pss);
            out.push(b' ');
            push_u64(&mut out, *rss);
            out.push(b'\n');
        }
        out.extend_from_slice(b"total ");
        push_u64(&mut out, tp);
        out.push(b' ');
        push_u64(&mut out, tr);
        out.push(b'\n');
        if args.is_empty() {
            let stdout = std::io::stdout();
            let mut o = stdout.lock();
            o.write_all(&out)?;
            o.flush()?;
        } else {
            let p = std::path::Path::new(args[0].as_os_str());
            if let Err(ee) = std::fs::write(p, &out) {
                eprintln!("smemcap: {}", ee);
                return Ok(1);
            }
        }
        Ok(0)
    }
}

pub struct ReadprofileApplet;
impl Applet for ReadprofileApplet {
    fn name(&self) -> &'static str {
        "readprofile"
    }
    fn description(&self) -> &'static str {
        "Read kernel profiling data"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if !args.is_empty() {
            eprintln!("usage: readprofile");
            return Ok(1);
        }

        let data = match std::fs::read("/proc/profile") {
            Ok(d) => d,
            Err(e) => {
                eprintln!("readprofile: /proc/profile: {}", e);
                return Ok(1);
            }
        };
        if data.len() < 12 {
            eprintln!("readprofile: short profile");
            return Ok(1);
        }

        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut addr: u64 = 0;
        let mut i = 4;
        while i + 4 <= data.len() {
            let hits = u32::from_ne_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as u64;
            if hits > 0 {
                let mut line = Vec::with_capacity(48);

                let mut v = addr;
                let mut rev = [0u8; 16];
                let mut rn = 0;
                if v == 0 {
                    rev[0] = b'0';
                    rn = 1;
                } else {
                    while v > 0 {
                        let d = (v & 0xf) as u8;
                        rev[rn] = if d < 10 { b'0' + d } else { b'a' + d - 10 };
                        v >>= 4;
                        rn += 1;
                    }
                }
                while rn > 0 {
                    rn -= 1;
                    line.push(rev[rn]);
                }
                line.push(b' ');
                push_u64(&mut line, hits);
                line.push(b'\n');
                out.write_all(&line)?;
            }
            addr += 4;
            i += 4;
        }
        out.flush()?;
        Ok(0)
    }
}

pub struct RdevApplet;
impl Applet for RdevApplet {
    fn name(&self) -> &'static str {
        "rdev"
    }
    fn description(&self) -> &'static str {
        "Print the root device"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if !args.is_empty() {
            eprintln!("rdev: setting devices is not supported");
            return Ok(1);
        }
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::stat(c"/".as_ptr(), &mut st) } != 0 {
            eprintln!("rdev: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        let maj = libc::major(st.st_dev);
        let min = libc::minor(st.st_dev);
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line = Vec::with_capacity(48);
        line.extend_from_slice(b"root device ");
        push_u64(&mut line, maj as u64);
        line.push(b':');
        push_u64(&mut line, min as u64);
        line.push(b'\n');
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}

pub struct FallocateApplet;
impl Applet for FallocateApplet {
    fn name(&self) -> &'static str {
        "fallocate"
    }
    fn description(&self) -> &'static str {
        "Preallocate space for a file"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut len: Option<u64> = None;
        let mut off: u64 = 0;
        let mut mode: i32 = 0;
        let mut file: Option<&[u8]> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-l" || b == b"--length" {
                i += 1;
                if i >= args.len() {
                    eprintln!("fallocate: -l needs a length");
                    return Ok(1);
                }
                match parse_u64_suffix(args[i].as_bytes()) {
                    Some(v) => len = Some(v),
                    None => {
                        eprintln!("fallocate: invalid length");
                        return Ok(1);
                    }
                }
            } else if b.starts_with(b"--length=") {
                match parse_u64_suffix(&b[9..]) {
                    Some(v) => len = Some(v),
                    None => {
                        eprintln!("fallocate: invalid length");
                        return Ok(1);
                    }
                }
            } else if b == b"-o" || b == b"--offset" {
                i += 1;
                if i >= args.len() {
                    eprintln!("fallocate: -o needs an offset");
                    return Ok(1);
                }
                match parse_u64_suffix(args[i].as_bytes()) {
                    Some(v) => off = v,
                    None => {
                        eprintln!("fallocate: invalid offset");
                        return Ok(1);
                    }
                }
            } else if b.starts_with(b"--offset=") {
                match parse_u64_suffix(&b[9..]) {
                    Some(v) => off = v,
                    None => {
                        eprintln!("fallocate: invalid offset");
                        return Ok(1);
                    }
                }
            } else if b == b"-n" || b == b"--keep-size" {
                mode |= libc::FALLOC_FL_KEEP_SIZE;
            } else if b.first() == Some(&b'-') {
                eprintln!("fallocate: unknown option");
                return Ok(1);
            } else if file.is_none() {
                file = Some(b);
            } else {
                eprintln!("fallocate: too many files");
                return Ok(1);
            }
            i += 1;
        }
        let (len, file) = match (len, file) {
            (Some(l), Some(f)) => (l, f),
            _ => {
                eprintln!("usage: fallocate [-o OFFSET] -l LENGTH [-n] FILE");
                return Ok(1);
            }
        };
        use std::ffi::CString;
        let cf = match CString::new(file) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("fallocate: bad filename");
                return Ok(1);
            }
        };
        let fd = unsafe { libc::open(cf.as_ptr(), libc::O_RDWR | libc::O_CREAT, 0o644) };
        if fd < 0 {
            eprintln!(
                "fallocate: '{}': {}",
                String::from_utf8_lossy(file),
                std::io::Error::last_os_error()
            );
            return Ok(1);
        }
        let r = unsafe { libc::fallocate(fd, mode, off as libc::off_t, len as libc::off_t) };
        unsafe { libc::close(fd) };
        if r != 0 {
            eprintln!("fallocate: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}

fn valid_runpart(name: &[u8]) -> bool {
    !name.is_empty()
        && name
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'-')
}

pub struct RunPartsApplet;
impl Applet for RunPartsApplet {
    fn name(&self) -> &'static str {
        "run-parts"
    }
    fn description(&self) -> &'static str {
        "Run scripts in a directory"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut test = false;
        let mut verbose = false;
        let mut reverse = false;
        let mut umask_v: Option<u32> = None;
        let mut pass_args: Vec<OsString> = Vec::new();
        let mut dir: Option<&[u8]> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-t" || b == b"--test" {
                test = true;
            } else if b == b"-v" || b == b"--verbose" {
                verbose = true;
            } else if b == b"--reverse" {
                reverse = true;
            } else if b == b"-a" {
                i += 1;
                if i >= args.len() {
                    eprintln!("run-parts: -a needs an argument");
                    return Ok(1);
                }
                pass_args.push(args[i].clone());
            } else if b == b"-u" || b == b"--umask" {
                i += 1;
                if i >= args.len() {
                    eprintln!("run-parts: -u needs a mask");
                    return Ok(1);
                }
                match u32::from_str_radix(std::str::from_utf8(args[i].as_bytes()).unwrap_or(""), 8)
                {
                    Ok(m) => umask_v = Some(m),
                    Err(_) => {
                        eprintln!("run-parts: invalid umask");
                        return Ok(1);
                    }
                }
            } else if b.first() == Some(&b'-') {
                eprintln!("run-parts: unknown option");
                return Ok(1);
            } else if dir.is_none() {
                dir = Some(b);
            } else {
                pass_args.push(args[i].clone());
            }
            i += 1;
        }
        let dir = match dir {
            Some(d) => d,
            None => {
                eprintln!("usage: run-parts [-t] [-v] [-a ARG] DIR");
                return Ok(1);
            }
        };
        if let Some(m) = umask_v {
            unsafe { libc::umask(m as libc::mode_t) };
        }
        let ds = match std::str::from_utf8(dir) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("run-parts: bad directory");
                return Ok(1);
            }
        };
        let mut entries: Vec<OsString> = Vec::new();
        let rd = match std::fs::read_dir(ds) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("run-parts: '{}': {}", ds, e);
                return Ok(1);
            }
        };
        for e in rd.flatten() {
            let n = e.file_name();
            if !valid_runpart(n.as_bytes()) {
                continue;
            }

            let p = e.path();
            let acc = {
                use std::ffi::CString;
                match CString::new(p.as_os_str().as_bytes()) {
                    Ok(c) => unsafe { libc::access(c.as_ptr(), libc::X_OK) == 0 },
                    Err(_) => false,
                }
            };
            if acc && p.is_file() {
                entries.push(n);
            }
        }
        entries.sort();
        if reverse {
            entries.reverse();
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut rc = 0;
        for n in &entries {
            let mut full = Vec::with_capacity(ds.len() + n.len() + 2);
            full.extend_from_slice(ds.as_bytes());
            full.push(b'/');
            full.extend_from_slice(n.as_bytes());
            if test || verbose {
                out.write_all(&full)?;
                out.write_all(b"\n")?;
            }
            if test {
                continue;
            }

            let pid = unsafe { libc::fork() };
            if pid < 0 {
                eprintln!("run-parts: fork: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            if pid == 0 {
                use std::ffi::CString;
                let prog = CString::new(full.clone()).unwrap();
                let mut cs: Vec<CString> = Vec::with_capacity(pass_args.len() + 2);
                cs.push(prog.clone());
                for a in &pass_args {
                    cs.push(
                        CString::new(a.as_bytes()).unwrap_or_else(|_| CString::new("").unwrap()),
                    );
                }
                let mut ptrs: Vec<*const libc::c_char> = cs.iter().map(|c| c.as_ptr()).collect();
                ptrs.push(std::ptr::null());
                unsafe {
                    libc::execv(prog.as_ptr(), ptrs.as_ptr());
                    libc::_exit(127);
                }
            }
            let mut st = 0;
            unsafe {
                while libc::waitpid(pid, &mut st, 0) < 0
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR)
                {
                }
                if !(libc::WIFEXITED(st) && libc::WEXITSTATUS(st) == 0) {
                    rc = 1;
                }
            }
        }
        out.flush()?;
        Ok(rc)
    }
}
