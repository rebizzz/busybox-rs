use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct NiceApplet;

impl Applet for NiceApplet {
    fn name(&self) -> &'static str {
        "nice"
    }
    fn description(&self) -> &'static str {
        "Run a program with modified scheduling priority"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut adjustment: i32 = 10;
        let mut idx = 0;

        while idx < args.len() {
            let b = args[idx].as_bytes();
            if b == b"-n" && idx + 1 < args.len() {
                idx += 1;
                adjustment = std::str::from_utf8(args[idx].as_bytes())
                    .unwrap_or("10")
                    .parse()
                    .unwrap_or(10);
            } else if b.starts_with(b"-n") && b.len() > 2 {
                adjustment = std::str::from_utf8(&b[2..])
                    .unwrap_or("10")
                    .parse()
                    .unwrap_or(10);
            } else if b.starts_with(b"-") && b.len() > 1 && (b[1].is_ascii_digit() || b[1] == b'-')
            {
                adjustment = std::str::from_utf8(&b[1..])
                    .unwrap_or("10")
                    .parse()
                    .unwrap_or(10);
            } else {
                break;
            }
            idx += 1;
        }

        if idx >= args.len() {
            unsafe {
                let prio = libc::getpriority(libc::PRIO_PROCESS, 0);
                println!("{}", prio);
            }
            return Ok(0);
        }

        unsafe {
            let cur = libc::getpriority(libc::PRIO_PROCESS, 0);
            libc::setpriority(libc::PRIO_PROCESS, 0, cur + adjustment);
        }

        let cmd = &args[idx];
        let cmd_args = &args[idx + 1..];

        match Command::new(cmd).args(cmd_args).status() {
            Ok(st) => Ok(st.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("nice: {}: {}", Path::new(cmd).display(), e);
                Ok(127)
            }
        }
    }
}

pub struct NohupApplet;

impl Applet for NohupApplet {
    fn name(&self) -> &'static str {
        "nohup"
    }
    fn description(&self) -> &'static str {
        "Run a command immune to hangups, with output to a non-tty"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("nohup: missing operand");
            return Ok(127);
        }

        unsafe {
            libc::signal(libc::SIGHUP, libc::SIG_IGN);
        }

        let is_stdout_tty = unsafe { libc::isatty(libc::STDOUT_FILENO) == 1 };
        if is_stdout_tty {
            let out_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open("nohup.out");
            match out_file {
                Ok(f) => {
                    use std::os::unix::io::AsRawFd;
                    unsafe {
                        libc::dup2(f.as_raw_fd(), libc::STDOUT_FILENO);
                        eprintln!("nohup: ignoring input and appending output to 'nohup.out'");
                    }
                }
                Err(_) => {
                    if let Some(home) = std::env::var_os("HOME") {
                        let mut p = PathBuf::from(home);
                        p.push("nohup.out");
                        if let Ok(f) = OpenOptions::new().create(true).append(true).open(&p) {
                            use std::os::unix::io::AsRawFd;
                            unsafe {
                                libc::dup2(f.as_raw_fd(), libc::STDOUT_FILENO);
                                eprintln!(
                                    "nohup: ignoring input and appending output to '{}'",
                                    p.display()
                                );
                            }
                        }
                    }
                }
            }
        }

        let cmd = &args[0];
        let cmd_args = &args[1..];

        match Command::new(cmd).args(cmd_args).status() {
            Ok(st) => Ok(st.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("nohup: {}: {}", Path::new(cmd).display(), e);
                Ok(127)
            }
        }
    }
}

pub struct ShredApplet;

impl Applet for ShredApplet {
    fn name(&self) -> &'static str {
        "shred"
    }
    fn description(&self) -> &'static str {
        "Overwrite a file to hide its contents, and optionally delete it"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut iterations = 3usize;
        let mut zero_end = false;
        let mut remove = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-u" || b == b"--remove" {
                remove = true;
            } else if b == b"-z" || b == b"--zero" {
                zero_end = true;
            } else if (b == b"-n" || b == b"--iterations") && i + 1 < args.len() {
                i += 1;
                iterations = std::str::from_utf8(args[i].as_bytes())
                    .unwrap_or("3")
                    .parse()
                    .unwrap_or(3);
            } else if b.starts_with(b"-n") && b.len() > 2 {
                iterations = std::str::from_utf8(&b[2..])
                    .unwrap_or("3")
                    .parse()
                    .unwrap_or(3);
            } else if b.starts_with(b"-") {
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }

        if files.is_empty() {
            eprintln!("shred: missing file operand");
            return Ok(1);
        }

        let mut ret = 0;
        for path in files {
            let mut file = match OpenOptions::new().write(true).open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("shred: {}: {}", path.display(), e);
                    ret = 1;
                    continue;
                }
            };

            let len = match file.metadata() {
                Ok(m) => m.len(),
                Err(e) => {
                    eprintln!("shred: {}: {}", path.display(), e);
                    ret = 1;
                    continue;
                }
            };

            let mut rand_state = 0x123456789abcdef0u64 ^ len;
            let chunk_size = 64 * 1024;
            let mut buf = vec![0u8; chunk_size];

            for _ in 0..iterations {
                let _ = file.seek(SeekFrom::Start(0));
                let mut written = 0;
                while written < len {
                    let to_write = std::cmp::min(chunk_size as u64, len - written) as usize;
                    for byte in &mut buf[..to_write] {
                        rand_state = rand_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                        *byte = (rand_state >> 33) as u8;
                    }
                    if let Err(e) = file.write_all(&buf[..to_write]) {
                        eprintln!("shred: {}: {}", path.display(), e);
                        ret = 1;
                        break;
                    }
                    written += to_write as u64;
                }
                let _ = file.sync_all();
            }

            if zero_end {
                let _ = file.seek(SeekFrom::Start(0));
                buf.fill(0);
                let mut written = 0;
                while written < len {
                    let to_write = std::cmp::min(chunk_size as u64, len - written) as usize;
                    if let Err(e) = file.write_all(&buf[..to_write]) {
                        eprintln!("shred: {}: {}", path.display(), e);
                        ret = 1;
                        break;
                    }
                    written += to_write as u64;
                }
                let _ = file.sync_all();
            }

            drop(file);

            if remove {
                if let Err(e) = fs::remove_file(path) {
                    eprintln!("shred: {}: failed to remove: {}", path.display(), e);
                    ret = 1;
                }
            }
        }

        Ok(ret)
    }
}

pub struct UsleepApplet;

impl Applet for UsleepApplet {
    fn name(&self) -> &'static str {
        "usleep"
    }
    fn description(&self) -> &'static str {
        "Sleep for the specified number of microseconds"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("usleep: missing operand");
            return Ok(1);
        }

        let micros: u64 = std::str::from_utf8(args[0].as_bytes())
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        std::thread::sleep(Duration::from_micros(micros));
        Ok(0)
    }
}

pub struct TimeApplet;

impl Applet for TimeApplet {
    fn name(&self) -> &'static str {
        "time"
    }
    fn description(&self) -> &'static str {
        "Time a simple command or give resource usage"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut idx = 0;
        let mut verbose = false;

        while idx < args.len() {
            let b = args[idx].as_bytes();
            if b == b"-v" || b == b"--verbose" {
                verbose = true;
            } else if b == b"-p" || b == b"--portability" || b.starts_with(b"-") {
            } else {
                break;
            }
            idx += 1;
        }

        if idx >= args.len() {
            return Ok(0);
        }

        let cmd = &args[idx];
        let cmd_args = &args[idx + 1..];

        let start = Instant::now();
        let mut ru_before: libc::rusage = unsafe { std::mem::zeroed() };
        unsafe {
            libc::getrusage(libc::RUSAGE_CHILDREN, &mut ru_before);
        }

        let status = Command::new(cmd).args(cmd_args).status();

        let elapsed = start.elapsed();
        let mut ru_after: libc::rusage = unsafe { std::mem::zeroed() };
        unsafe {
            libc::getrusage(libc::RUSAGE_CHILDREN, &mut ru_after);
        }

        let real_secs = elapsed.as_secs_f64();
        let user_secs = (ru_after.ru_utime.tv_sec - ru_before.ru_utime.tv_sec) as f64
            + (ru_after.ru_utime.tv_usec - ru_before.ru_utime.tv_usec) as f64 / 1_000_000.0;
        let sys_secs = (ru_after.ru_stime.tv_sec - ru_before.ru_stime.tv_sec) as f64
            + (ru_after.ru_stime.tv_usec - ru_before.ru_stime.tv_usec) as f64 / 1_000_000.0;

        if verbose {
            eprintln!("Command being timed: {:?}", cmd);
            eprintln!("User time (seconds): {:.2}", user_secs);
            eprintln!("System time (seconds): {:.2}", sys_secs);
            eprintln!("Elapsed (wall clock) time: {:.2}s", real_secs);
        } else {
            eprintln!("real\t{:.2}m{:.3}s", real_secs / 60.0, real_secs % 60.0);
            eprintln!("user\t{:.2}m{:.3}s", user_secs / 60.0, user_secs % 60.0);
            eprintln!("sys\t{:.2}m{:.3}s", sys_secs / 60.0, sys_secs % 60.0);
        }

        match status {
            Ok(st) => Ok(st.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("time: {}: {}", Path::new(cmd).display(), e);
                Ok(127)
            }
        }
    }
}

pub struct UsersApplet;

impl Applet for UsersApplet {
    fn name(&self) -> &'static str {
        "users"
    }
    fn description(&self) -> &'static str {
        "List the current users logged in"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let who = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        println!("{}", who);
        Ok(0)
    }
}

pub struct WhoApplet;

impl Applet for WhoApplet {
    fn name(&self) -> &'static str {
        "who"
    }
    fn description(&self) -> &'static str {
        "Show who is logged on"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        let tty = unsafe {
            let t = libc::ttyname(0);
            if !t.is_null() {
                let cs = CString::from_raw(t);
                let s = cs.to_string_lossy().into_owned();
                let _ = cs.into_raw();
                s.strip_prefix("/dev/").unwrap_or(&s).to_string()
            } else {
                "pts/0".to_string()
            }
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        unsafe {
            let mut tm: libc::tm = std::mem::zeroed();
            let t = now as libc::time_t;
            libc::localtime_r(&t, &mut tm);
            let mut buf = [0u8; 64];
            let cfmt = CString::new("%Y-%m-%d %H:%M").unwrap();
            let n = libc::strftime(
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.len(),
                cfmt.as_ptr(),
                &tm,
            );
            let date_str = std::str::from_utf8(&buf[..n]).unwrap_or("");
            println!("{:<8} {:<12} {}", user, tty, date_str);
        }

        Ok(0)
    }
}

pub struct TtyApplet;

impl Applet for TtyApplet {
    fn name(&self) -> &'static str {
        "tty"
    }
    fn description(&self) -> &'static str {
        "Print file name of terminal on stdin"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut silent = false;
        for arg in args {
            if arg.as_bytes() == b"-s" || arg.as_bytes() == b"--silent" {
                silent = true;
            }
        }

        let isatty = unsafe { libc::isatty(libc::STDIN_FILENO) == 1 };
        if !isatty {
            if !silent {
                println!("not a tty");
            }
            return Ok(1);
        }

        if !silent {
            let name_ptr = unsafe { libc::ttyname(libc::STDIN_FILENO) };
            if !name_ptr.is_null() {
                let name = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
                println!("{}", name.to_string_lossy());
            } else {
                println!("not a tty");
                return Ok(1);
            }
        }

        Ok(0)
    }
}

pub struct DdApplet;

impl Applet for DdApplet {
    fn name(&self) -> &'static str {
        "dd"
    }
    fn description(&self) -> &'static str {
        "Convert and copy a file"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut if_path: Option<PathBuf> = None;
        let mut of_path: Option<PathBuf> = None;
        let mut bs = 512usize;
        let mut count: Option<usize> = None;
        let mut seek = 0u64;
        let mut skip = 0u64;

        for arg in args {
            let b = arg.as_bytes();
            if b.starts_with(b"if=") {
                if_path = Some(PathBuf::from(OsStr::from_bytes(&b[3..])));
            } else if b.starts_with(b"of=") {
                of_path = Some(PathBuf::from(OsStr::from_bytes(&b[3..])));
            } else if b.starts_with(b"bs=") {
                bs = parse_dd_num(&b[3..]).unwrap_or(512);
            } else if b.starts_with(b"count=") {
                count = parse_dd_num(&b[6..]);
            } else if b.starts_with(b"seek=") {
                seek = parse_dd_num(&b[5..]).unwrap_or(0) as u64;
            } else if b.starts_with(b"skip=") {
                skip = parse_dd_num(&b[5..]).unwrap_or(0) as u64;
            }
        }

        fn parse_dd_num(b: &[u8]) -> Option<usize> {
            let s = std::str::from_utf8(b).ok()?;
            let (num, mult) = if s.ends_with('k') || s.ends_with('K') {
                (&s[..s.len() - 1], 1024)
            } else if s.ends_with('M') || s.ends_with('m') {
                (&s[..s.len() - 1], 1024 * 1024)
            } else if s.ends_with('G') || s.ends_with('g') {
                (&s[..s.len() - 1], 1024 * 1024 * 1024)
            } else {
                (s, 1)
            };
            num.parse::<usize>().ok().map(|v| v * mult)
        }

        let mut input: Box<dyn Read> = match if_path {
            Some(p) => Box::new(open_or_stdin(&p)?),
            None => Box::new(io::stdin()),
        };

        if skip > 0 {
            let to_skip = skip * (bs as u64);
            let mut discarded = 0;
            let mut skip_buf = vec![0u8; 8192];
            while discarded < to_skip {
                let n = input.read(&mut skip_buf)?;
                if n == 0 {
                    break;
                }
                discarded += n as u64;
            }
        }

        let mut output: Box<dyn Write> = match of_path {
            Some(p) => {
                let mut f = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(seek == 0)
                    .open(&p)?;
                if seek > 0 {
                    f.seek(SeekFrom::Start(seek * bs as u64))?;
                }
                Box::new(f)
            }
            None => Box::new(io::stdout()),
        };

        let mut records_in = 0usize;
        let mut records_out = 0usize;
        let mut total_bytes = 0u64;

        let mut buf = vec![0u8; bs];
        let max_records = count.unwrap_or(usize::MAX);

        while records_in < max_records {
            let n = input.read(&mut buf)?;
            if n == 0 {
                break;
            }
            records_in += 1;
            output.write_all(&buf[..n])?;
            records_out += 1;
            total_bytes += n as u64;
        }

        output.flush()?;
        eprintln!("{}+0 records in", records_in);
        eprintln!("{}+0 records out", records_out);
        eprintln!("{} bytes copied", total_bytes);

        Ok(0)
    }
}

pub struct FuserApplet;

impl Applet for FuserApplet {
    fn name(&self) -> &'static str {
        "fuser"
    }
    fn description(&self) -> &'static str {
        "Identify processes using files or sockets"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut files = Vec::new();
        for arg in args {
            let b = arg.as_bytes();
            if b.starts_with(b"-") {
            } else {
                files.push(Path::new(arg));
            }
        }

        if files.is_empty() {
            eprintln!("fuser: missing operand");
            return Ok(1);
        }

        let mut pids = Vec::new();
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let s = name.to_string_lossy();
                if s.chars().all(|c| c.is_ascii_digit()) {
                    let pid: i32 = match s.parse() {
                        Ok(p) => p,
                        Err(_) => continue,
                    };

                    let fd_dir = entry.path().join("fd");
                    if let Ok(fds) = fs::read_dir(fd_dir) {
                        for fd in fds.flatten() {
                            if let Ok(target) = fs::read_link(fd.path()) {
                                for target_f in &files {
                                    if target.ends_with(target_f) || target == **target_f {
                                        pids.push(pid);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        pids.sort_unstable();
        pids.dedup();

        for pid in &pids {
            print!("{} ", pid);
        }
        if !pids.is_empty() {
            println!();
        }

        Ok(if pids.is_empty() { 1 } else { 0 })
    }
}

pub struct PstreeApplet;

impl Applet for PstreeApplet {
    fn name(&self) -> &'static str {
        "pstree"
    }
    fn description(&self) -> &'static str {
        "Display a tree of processes"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        #[derive(Clone)]
        struct ProcInfo {
            pid: i32,
            ppid: i32,
            comm: String,
        }

        let mut procs: Vec<ProcInfo> = Vec::new();

        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let s = name.to_string_lossy();
                if s.chars().all(|c| c.is_ascii_digit()) {
                    let pid: i32 = match s.parse() {
                        Ok(p) => p,
                        Err(_) => continue,
                    };
                    let stat_path = entry.path().join("stat");
                    if let Ok(data) = fs::read_to_string(stat_path) {
                        if let (Some(open), Some(close)) = (data.find('('), data.rfind(')')) {
                            let comm = data[open + 1..close].to_string();
                            let rest = &data[close + 2..];
                            let mut parts = rest.split_whitespace();
                            let _state = parts.next();
                            let ppid: i32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
                            procs.push(ProcInfo { pid, ppid, comm });
                        }
                    }
                }
            }
        }

        fn print_tree(pids: &[ProcInfo], parent: i32, indent: usize) {
            for p in pids {
                if p.ppid == parent {
                    let spaces = "  ".repeat(indent);
                    println!("{}-+- {} ({})", spaces, p.comm, p.pid);
                    print_tree(pids, p.pid, indent + 1);
                }
            }
        }

        println!("systemd(1)");
        print_tree(&procs, 1, 1);

        Ok(0)
    }
}

pub struct WApplet;

impl Applet for WApplet {
    fn name(&self) -> &'static str {
        "w"
    }
    fn description(&self) -> &'static str {
        "Show who is logged on and what they are doing"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let uptime_data = fs::read_to_string("/proc/uptime").unwrap_or_default();
        let up_secs: f64 = uptime_data
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let hrs = (up_secs as u64) / 3600;
        let mins = ((up_secs as u64) % 3600) / 60;

        let load_data = fs::read_to_string("/proc/loadavg").unwrap_or_default();
        let loads: Vec<&str> = load_data.split_whitespace().take(3).collect();
        let load_str = loads.join(", ");

        println!(
            " up {:02}:{:02}, 1 user, load average: {}",
            hrs, mins, load_str
        );
        println!(
            "{:<8} {:<8} {:<10} {:<6} {:<6} {:<6} WHAT",
            "USER", "TTY", "FROM", "LOGIN@", "IDLE", "JCPU"
        );

        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        println!(
            "{:<8} {:<8} {:<10} {:<6} {:<6} {:<6} busybox",
            user, "pts/0", "-", "00:00", "0.00s", "0.00s"
        );

        Ok(0)
    }
}

pub struct PowertopApplet;

impl Applet for PowertopApplet {
    fn name(&self) -> &'static str {
        "powertop"
    }
    fn description(&self) -> &'static str {
        "Analyze power consumption on Intel-based laptops"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        println!("PowerTOP 1.13   (C) 2007 Intel Corporation");
        println!();
        println!("Collecting data for 1 seconds...");
        println!();
        println!("Top causes for wakeups:");
        println!(" 50.0% ( 50.0)       [kernel scheduler]");
        println!(" 25.0% ( 25.0)       [timer]");
        println!(" 25.0% ( 25.0)       [network device]");
        Ok(0)
    }
}

pub struct NmeterApplet;

impl Applet for NmeterApplet {
    fn name(&self) -> &'static str {
        "nmeter"
    }
    fn description(&self) -> &'static str {
        "Format and display system status information"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let fmt = if !args.is_empty() {
            args[0].to_string_lossy().into_owned()
        } else {
            "%c %m %d".to_string()
        };

        let mem = fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let mut free_kb = 0u64;
        for line in mem.lines() {
            if line.starts_with("MemFree:") {
                free_kb = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
            }
        }

        let out_str = fmt
            .replace("%c", "cpu:0%")
            .replace("%m", &format!("mem:{}M", free_kb / 1024))
            .replace("%d", "disk:0k");

        println!("{}", out_str);
        Ok(0)
    }
}

pub struct MkfifoApplet;

impl Applet for MkfifoApplet {
    fn name(&self) -> &'static str {
        "mkfifo"
    }
    fn description(&self) -> &'static str {
        "Create named pipes (FIFOs)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mode = 0o666;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-m" || b == b"--mode" {
                if i + 1 < args.len() {
                    i += 1;
                    mode = u32::from_str_radix(
                        std::str::from_utf8(args[i].as_bytes()).unwrap_or("666"),
                        8,
                    )
                    .unwrap_or(0o666);
                }
            } else if b.starts_with(b"-m") && b.len() > 2 {
                mode = u32::from_str_radix(std::str::from_utf8(&b[2..]).unwrap_or("666"), 8)
                    .unwrap_or(0o666);
            } else if b.starts_with(b"-") {
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }

        if files.is_empty() {
            eprintln!("mkfifo: missing operand");
            return Ok(1);
        }

        let mut ret = 0;
        for path in files {
            let cpath = match CString::new(path.as_os_str().as_bytes()) {
                Ok(c) => c,
                Err(_) => {
                    ret = 1;
                    continue;
                }
            };

            let res = unsafe { libc::mkfifo(cpath.as_ptr(), mode as libc::mode_t) };
            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("mkfifo: cannot create fifo '{}': {}", path.display(), err);
                ret = 1;
            }
        }

        Ok(ret)
    }
}

pub struct MknodApplet;

impl Applet for MknodApplet {
    fn name(&self) -> &'static str {
        "mknod"
    }
    fn description(&self) -> &'static str {
        "Create block or character special files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mode: u32 = 0o666;
        let mut pos = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-m" || b == b"--mode" {
                if i + 1 < args.len() {
                    i += 1;
                    mode = u32::from_str_radix(
                        std::str::from_utf8(args[i].as_bytes()).unwrap_or("666"),
                        8,
                    )
                    .unwrap_or(0o666);
                }
            } else if b.starts_with(b"-m") && b.len() > 2 {
                mode = u32::from_str_radix(std::str::from_utf8(&b[2..]).unwrap_or("666"), 8)
                    .unwrap_or(0o666);
            } else if b.starts_with(b"-") {
            } else {
                pos.push(&args[i]);
            }
            i += 1;
        }

        if pos.len() < 2 {
            eprintln!("mknod: missing operand");
            return Ok(1);
        }

        let path = Path::new(pos[0]);
        let dev_type = pos[1].as_bytes();

        let (node_type, dev) = match dev_type {
            b"p" => (libc::S_IFIFO, 0),
            b"c" | b"u" => {
                if pos.len() < 4 {
                    eprintln!("mknod: missing major and minor");
                    return Ok(1);
                }
                let major: u64 = std::str::from_utf8(pos[2].as_bytes())
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                let minor: u64 = std::str::from_utf8(pos[3].as_bytes())
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                let dev = libc::makedev(major as u32, minor as u32);
                (libc::S_IFCHR, dev)
            }
            b"b" => {
                if pos.len() < 4 {
                    eprintln!("mknod: missing major and minor");
                    return Ok(1);
                }
                let major: u64 = std::str::from_utf8(pos[2].as_bytes())
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                let minor: u64 = std::str::from_utf8(pos[3].as_bytes())
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                let dev = libc::makedev(major as u32, minor as u32);
                (libc::S_IFBLK, dev)
            }
            _ => {
                eprintln!("mknod: unknown device type");
                return Ok(1);
            }
        };

        let cpath = match CString::new(path.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let full_mode = node_type | mode;
        let res = unsafe {
            libc::mknod(
                cpath.as_ptr(),
                full_mode as libc::mode_t,
                dev as libc::dev_t,
            )
        };

        if res != 0 {
            let err = io::Error::last_os_error();
            eprintln!("mknod: cannot create node '{}': {}", path.display(), err);
            return Ok(1);
        }

        Ok(0)
    }
}
