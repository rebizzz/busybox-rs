use crate::core::{Applet, Result};
use std::env;
use std::ffi::{CString, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::mem::MaybeUninit;
use std::net::TcpStream;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const FIFREEZE: libc::c_ulong = 0xc0045877;
const FITHAW: libc::c_ulong = 0xc0045878;
const BLKRRPART: libc::c_ulong = 0x125f;
const VT_OPENQRY: libc::c_ulong = 0x5600;
const VT_ACTIVATE: libc::c_ulong = 0x5606;
const VT_WAITACTIVE: libc::c_ulong = 0x5607;
const TIOCGWINSZ: libc::c_ulong = 0x5413;
const TIOCLINUX: libc::c_ulong = 0x541c;
const TIOCGSERIAL: libc::c_ulong = 0x541e;
const TIOCSSERIAL: libc::c_ulong = 0x541f;

const NBD_SET_SOCK: libc::c_ulong = 0xab00;
const NBD_SET_BLKSIZE: libc::c_ulong = 0xab01;
const NBD_DO_IT: libc::c_ulong = 0xab03;
const NBD_CLEAR_SOCK: libc::c_ulong = 0xab04;
const NBD_CLEAR_QUE: libc::c_ulong = 0xab05;
const NBD_DISCONNECT: libc::c_ulong = 0xab08;
const NBD_SET_TIMEOUT: libc::c_ulong = 0xab09;

#[repr(C)]
struct SerialStruct {
    type_: libc::c_int,
    line: libc::c_int,
    port: libc::c_uint,
    irq: libc::c_int,
    flags: libc::c_int,
    xmit_fifo_size: libc::c_int,
    custom_divisor: libc::c_int,
    baud_base: libc::c_int,
    close_delay: libc::c_ushort,
    io_type: libc::c_char,
    reserved_char: libc::c_char,
    hub6: libc::c_int,
    closing_wait: libc::c_ushort,
    closing_wait2: libc::c_ushort,
    iomem_base: *mut libc::c_void,
    iomem_reg_shift: libc::c_ushort,
    port_high: libc::c_uint,
    iomap_base: libc::c_ulong,
}

#[repr(C)]
struct Winsize {
    ws_row: libc::c_ushort,
    ws_col: libc::c_ushort,
    ws_xpixel: libc::c_ushort,
    ws_ypixel: libc::c_ushort,
}

pub struct SysctlApplet;
impl Applet for SysctlApplet {
    fn name(&self) -> &'static str {
        "sysctl"
    }
    fn description(&self) -> &'static str {
        "Configure kernel parameters at runtime"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut show_all = false;
        let mut write_mode = false;
        let mut quiet = false;
        let mut targets = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-a" || b == b"-A" {
                show_all = true;
            } else if b == b"-w" {
                write_mode = true;
            } else if b == b"-n" || b == b"-q" {
                quiet = true;
            } else if b == b"-p" {
                let conf_path = if i + 1 < args.len() && !args[i + 1].as_bytes().starts_with(b"-") {
                    i += 1;
                    PathBuf::from(&args[i])
                } else {
                    PathBuf::from("/etc/sysctl.conf")
                };
                if let Ok(file) = File::open(&conf_path) {
                    for line in BufReader::new(file)
                        .lines()
                        .map_while(std::result::Result::ok)
                    {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                            continue;
                        }
                        if let Some((k, v)) = line.split_once('=') {
                            let k = k.trim();
                            let v = v.trim();
                            write_sysctl(k, v, quiet);
                        }
                    }
                }
                return Ok(0);
            } else if !b.starts_with(b"-") {
                targets.push(args[i].to_string_lossy().to_string());
            }
            i += 1;
        }

        if show_all {
            dump_sysctl_dir(Path::new("/proc/sys"), "");
            return Ok(0);
        }

        if targets.is_empty() {
            eprintln!("sysctl: no variables specified");
            return Ok(1);
        }

        let mut status = 0;
        for target in targets {
            if target.contains('=') || write_mode {
                let (k, v) = match target.split_once('=') {
                    Some((k, v)) => (k.trim(), v.trim()),
                    None => {
                        eprintln!("sysctl: key=val required for -w");
                        status = 1;
                        continue;
                    }
                };
                if !write_sysctl(k, v, quiet) {
                    status = 1;
                }
            } else {
                let proc_path = sysctl_to_proc(&target);
                match fs::read_to_string(&proc_path) {
                    Ok(val) => {
                        let val = val.trim();
                        if quiet {
                            println!("{}", val);
                        } else {
                            println!("{} = {}", target, val);
                        }
                    }
                    Err(e) => {
                        eprintln!("sysctl: error reading key '{}': {}", target, e);
                        status = 1;
                    }
                }
            }
        }

        Ok(status)
    }
}

fn sysctl_to_proc(key: &str) -> PathBuf {
    let sub = key.replace('.', "/");
    Path::new("/proc/sys").join(sub)
}

fn write_sysctl(key: &str, val: &str, quiet: bool) -> bool {
    let proc_path = sysctl_to_proc(key);
    match fs::write(&proc_path, format!("{}\n", val)) {
        Ok(_) => {
            if !quiet {
                println!("{} = {}", key, val);
            }
            true
        }
        Err(e) => {
            eprintln!("sysctl: setting key '{}': {}", key, e);
            false
        }
    }
}

fn dump_sysctl_dir(dir: &Path, prefix: &str) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let key = if prefix.is_empty() {
                name
            } else {
                format!("{}.{}", prefix, name)
            };
            if path.is_dir() {
                dump_sysctl_dir(&path, &key);
            } else if path.is_file() {
                if let Ok(val) = fs::read_to_string(&path) {
                    let val = val.trim();
                    println!("{} = {}", key, val);
                }
            }
        }
    }
}

pub struct SttyApplet;
impl Applet for SttyApplet {
    fn name(&self) -> &'static str {
        "stty"
    }
    fn description(&self) -> &'static str {
        "Change and print terminal line settings"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut termios = MaybeUninit::<libc::termios>::uninit();
        let fd = libc::STDIN_FILENO;

        if unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) } != 0 {
            eprintln!("stty: standard input: Inappropriate ioctl for device");
            return Ok(1);
        }
        let mut termios = unsafe { termios.assume_init() };

        if args.is_empty() {
            println!("speed 38400 baud; line = 0;");
            return Ok(0);
        }

        let mut changed = false;
        for arg in args {
            let s = arg.to_string_lossy();
            match s.as_ref() {
                "-a" | "--all" => {
                    let mut ws = MaybeUninit::<Winsize>::uninit();
                    let (rows, cols) =
                        if unsafe { libc::ioctl(fd, TIOCGWINSZ, ws.as_mut_ptr()) } == 0 {
                            let ws = unsafe { ws.assume_init() };
                            (ws.ws_row, ws.ws_col)
                        } else {
                            (24, 80)
                        };
                    println!(
                        "speed 38400 baud; rows {}; columns {}; line = 0;",
                        rows, cols
                    );
                    return Ok(0);
                }
                "size" => {
                    let mut ws = MaybeUninit::<Winsize>::uninit();
                    if unsafe { libc::ioctl(fd, TIOCGWINSZ, ws.as_mut_ptr()) } == 0 {
                        let ws = unsafe { ws.assume_init() };
                        println!("{} {}", ws.ws_row, ws.ws_col);
                    } else {
                        println!("24 80");
                    }
                    return Ok(0);
                }
                "raw" => {
                    unsafe { libc::cfmakeraw(&mut termios) };
                    changed = true;
                }
                "echo" => {
                    termios.c_lflag |= libc::ECHO;
                    changed = true;
                }
                "-echo" => {
                    termios.c_lflag &= !libc::ECHO;
                    changed = true;
                }
                "icanon" => {
                    termios.c_lflag |= libc::ICANON;
                    changed = true;
                }
                "-icanon" => {
                    termios.c_lflag &= !libc::ICANON;
                    changed = true;
                }
                "isig" => {
                    termios.c_lflag |= libc::ISIG;
                    changed = true;
                }
                "-isig" => {
                    termios.c_lflag &= !libc::ISIG;
                    changed = true;
                }
                _ => {}
            }
        }

        if changed {
            unsafe {
                libc::tcsetattr(fd, libc::TCSANOW, &termios);
            }
        }

        Ok(0)
    }
}

pub struct UnshareApplet;
impl Applet for UnshareApplet {
    fn name(&self) -> &'static str {
        "unshare"
    }
    fn description(&self) -> &'static str {
        "Run program with some namespaces unshared from parent"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut flags: libc::c_int = 0;
        let mut cmd_idx = args.len();

        for (i, arg) in args.iter().enumerate() {
            let b = arg.as_bytes();
            if b == b"-m" || b == b"--mount" {
                flags |= libc::CLONE_NEWNS;
            } else if b == b"-u" || b == b"--uts" {
                flags |= libc::CLONE_NEWUTS;
            } else if b == b"-i" || b == b"--ipc" {
                flags |= libc::CLONE_NEWIPC;
            } else if b == b"-n" || b == b"--net" {
                flags |= libc::CLONE_NEWNET;
            } else if b == b"-p" || b == b"--pid" {
                flags |= libc::CLONE_NEWPID;
            } else if b == b"-U" || b == b"--user" {
                flags |= libc::CLONE_NEWUSER;
            } else if !b.starts_with(b"-") {
                cmd_idx = i;
                break;
            }
        }

        if flags != 0 {
            let res = unsafe { libc::unshare(flags) };
            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("unshare: {}", err);
                return Ok(1);
            }
        }

        let cmd_parts = if cmd_idx < args.len() {
            &args[cmd_idx..]
        } else {
            &[]
        };

        let prog = if !cmd_parts.is_empty() {
            cmd_parts[0].to_string_lossy().to_string()
        } else {
            env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into())
        };

        let mut cmd = Command::new(&prog);
        if cmd_parts.len() > 1 {
            cmd.args(&cmd_parts[1..]);
        }

        match cmd.status() {
            Ok(st) => Ok(st.code().unwrap_or(0)),
            Err(e) => {
                eprintln!("unshare: {}: {}", prog, e);
                Ok(127)
            }
        }
    }
}

pub struct NsenterApplet;
impl Applet for NsenterApplet {
    fn name(&self) -> &'static str {
        "nsenter"
    }
    fn description(&self) -> &'static str {
        "Enter name space of other processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut target_pid: Option<libc::pid_t> = None;
        let mut nstypes: Vec<(&'static str, libc::c_int)> = Vec::new();
        let mut cmd_idx = args.len();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if (b == b"-t" || b == b"--target") && i + 1 < args.len() {
                target_pid = args[i + 1].to_string_lossy().parse().ok();
                i += 2;
                continue;
            } else if b == b"-m" || b == b"--mount" {
                nstypes.push(("mnt", libc::CLONE_NEWNS));
            } else if b == b"-u" || b == b"--uts" {
                nstypes.push(("uts", libc::CLONE_NEWUTS));
            } else if b == b"-i" || b == b"--ipc" {
                nstypes.push(("ipc", libc::CLONE_NEWIPC));
            } else if b == b"-n" || b == b"--net" {
                nstypes.push(("net", libc::CLONE_NEWNET));
            } else if b == b"-p" || b == b"--pid" {
                nstypes.push(("pid", libc::CLONE_NEWPID));
            } else if b == b"-U" || b == b"--user" {
                nstypes.push(("user", libc::CLONE_NEWUSER));
            } else if !b.starts_with(b"-") {
                cmd_idx = i;
                break;
            }
            i += 1;
        }

        let pid = match target_pid {
            Some(p) => p,
            None => {
                eprintln!("nsenter: -t PID required");
                return Ok(1);
            }
        };

        for (ns_name, clone_flag) in nstypes {
            let ns_path = format!("/proc/{}/ns/{}", pid, ns_name);
            let c_path = CString::new(ns_path).unwrap();
            let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
            if fd < 0 {
                let err = io::Error::last_os_error();
                eprintln!("nsenter: open /proc/{}/ns/{}: {}", pid, ns_name, err);
                return Ok(1);
            }
            let res = unsafe { libc::setns(fd, clone_flag) };
            unsafe { libc::close(fd) };
            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("nsenter: setns: {}", err);
                return Ok(1);
            }
        }

        let cmd_parts = if cmd_idx < args.len() {
            &args[cmd_idx..]
        } else {
            &[]
        };

        let prog = if !cmd_parts.is_empty() {
            cmd_parts[0].to_string_lossy().to_string()
        } else {
            env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into())
        };

        let mut cmd = Command::new(&prog);
        if cmd_parts.len() > 1 {
            cmd.args(&cmd_parts[1..]);
        }

        match cmd.status() {
            Ok(st) => Ok(st.code().unwrap_or(0)),
            Err(e) => {
                eprintln!("nsenter: {}: {}", prog, e);
                Ok(127)
            }
        }
    }
}

pub struct LpdApplet;
impl Applet for LpdApplet {
    fn name(&self) -> &'static str {
        "lpd"
    }
    fn description(&self) -> &'static str {
        "Line printer spooling daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let spool_dir = if !args.is_empty() {
            PathBuf::from(&args[0])
        } else {
            PathBuf::from("/var/spool/lpd")
        };

        if !spool_dir.exists() {
            let _ = fs::create_dir_all(&spool_dir);
        }

        let mut stdin = io::stdin().lock();
        let mut line = String::new();
        if stdin.read_line(&mut line).is_ok() {
            let _cmd = line.trim();

            let mut out = io::stdout().lock();
            let _ = out.write_all(b"\0");
            let _ = out.flush();
        }

        Ok(0)
    }
}

pub struct LpqApplet;
impl Applet for LpqApplet {
    fn name(&self) -> &'static str {
        "lpq"
    }
    fn description(&self) -> &'static str {
        "Spool queue examination program"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut printer = "lp";
        for (i, arg) in args.iter().enumerate() {
            if arg.as_bytes() == b"-P" && i + 1 < args.len() {
                printer = args[i + 1].to_str().unwrap_or("lp");
            }
        }

        let spool_dir = Path::new("/var/spool/lpd").join(printer);
        println!("Printer: {}", printer);
        if !spool_dir.exists() {
            println!("no entries");
            return Ok(0);
        }

        let mut found = false;
        if let Ok(entries) = fs::read_dir(spool_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("df") || name.starts_with("cf") {
                    println!("Job: {}", name);
                    found = true;
                }
            }
        }

        if !found {
            println!("no entries");
        }

        Ok(0)
    }
}

pub struct LprApplet;
impl Applet for LprApplet {
    fn name(&self) -> &'static str {
        "lpr"
    }
    fn description(&self) -> &'static str {
        "Send files to a print spooling daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut printer = "lp";
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-P" && i + 1 < args.len() {
                printer = args[i + 1].to_str().unwrap_or("lp");
                i += 2;
                continue;
            } else if !b.starts_with(b"-") {
                files.push(args[i].clone());
            }
            i += 1;
        }

        let spool_dir = Path::new("/var/spool/lpd").join(printer);
        let _ = fs::create_dir_all(&spool_dir);

        let job_id = unsafe { libc::getpid() };
        let df_name = format!("dfA{:03}localhost", job_id % 1000);
        let cf_name = format!("cfA{:03}localhost", job_id % 1000);

        let df_path = spool_dir.join(df_name);
        let cf_path = spool_dir.join(cf_name);

        let mut data_out = match File::create(&df_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("lpr: {}: {}", df_path.display(), e);
                return Ok(1);
            }
        };

        if files.is_empty() {
            let mut stdin = io::stdin().lock();
            let _ = io::copy(&mut stdin, &mut data_out);
        } else {
            for f in files {
                if let Ok(mut src) = File::open(&f) {
                    let _ = io::copy(&mut src, &mut data_out);
                }
            }
        }

        let _ = fs::write(&cf_path, "Hlocalhost\nPuser\nJjob\n");

        Ok(0)
    }
}

pub struct SvokApplet;
impl Applet for SvokApplet {
    fn name(&self) -> &'static str {
        "svok"
    }
    fn description(&self) -> &'static str {
        "Check whether runsv supervisor is running"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("svok: SERVICE_DIR required");
            return Ok(111);
        }

        let target = &args[0];
        let service_dir = Path::new(target);
        if !service_dir.exists() {
            eprintln!("svok: {}: file does not exist", service_dir.display());
            return Ok(111);
        }

        let ok_pipe = service_dir.join("supervise/ok");
        let c_path = match CString::new(ok_pipe.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(111),
        };

        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_WRONLY | libc::O_NONBLOCK) };
        if fd >= 0 {
            unsafe { libc::close(fd) };
            Ok(0)
        } else {
            let err = io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::ENXIO) || err.raw_os_error() == Some(libc::ENOENT) {
                Ok(100)
            } else {
                Ok(111)
            }
        }
    }
}

pub struct DumpleasesApplet;
impl Applet for DumpleasesApplet {
    fn name(&self) -> &'static str {
        "dumpleases"
    }
    fn description(&self) -> &'static str {
        "Display DHCP leases granted by udhcpd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut lease_file = "/var/lib/misc/udhcpd.leases";
        let mut show_abs = false;
        let mut show_remaining = false;
        let mut show_sec = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-f" && i + 1 < args.len() {
                lease_file = args[i + 1].to_str().unwrap_or(lease_file);
                i += 2;
                continue;
            } else if b == b"-a" {
                show_abs = true;
            } else if b == b"-r" {
                show_remaining = true;
            } else if b == b"-d" {
                show_sec = true;
            }
            i += 1;
        }

        let mut file = match File::open(lease_file) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("dumpleases: {}: {}", lease_file, e);
                return Ok(1);
            }
        };

        println!(
            "Mac Address       IP Address      Host Name       Expires {}",
            if show_abs { "at" } else { "in" }
        );

        let mut written_at_buf = [0u8; 8];
        if file.read_exact(&mut written_at_buf).is_err() {
            return Ok(0);
        }
        let written_at = u64::from_be_bytes(written_at_buf);
        let curr_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut lease_buf = [0u8; 36];
        while file.read_exact(&mut lease_buf).is_ok() {
            let expires_raw = u32::from_be_bytes(lease_buf[0..4].try_into().unwrap());
            let nip = u32::from_ne_bytes(lease_buf[4..8].try_into().unwrap());
            let mac = &lease_buf[8..14];
            let host_bytes = &lease_buf[14..34];
            let host_len = host_bytes.iter().position(|&b| b == 0).unwrap_or(20);
            let hostname = String::from_utf8_lossy(&host_bytes[..host_len]);

            let ip_octets = nip.to_le_bytes();
            let ip_str = format!(
                "{}.{}.{}.{}",
                ip_octets[0], ip_octets[1], ip_octets[2], ip_octets[3]
            );
            let mac_str = format!(
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
            );

            let expires_abs = expires_raw as u64 + written_at;
            let exp_str = if expires_abs <= curr_time {
                "expired".to_string()
            } else if show_sec {
                format!("{}", expires_abs - curr_time)
            } else if show_remaining || !show_abs {
                let rem = expires_abs - curr_time;
                format!("{:02}:{:02}:{:02}", rem / 3600, (rem % 3600) / 60, rem % 60)
            } else {
                format!("{}", expires_abs)
            };

            println!(
                "{:<17} {:<15} {:<15} {}",
                mac_str, ip_str, hostname, exp_str
            );
        }

        Ok(0)
    }
}

pub struct MkswapApplet;
impl Applet for MkswapApplet {
    fn name(&self) -> &'static str {
        "mkswap"
    }
    fn description(&self) -> &'static str {
        "Set up a Linux swap area"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut label = [0u8; 16];
        let mut target = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-L" && i + 1 < args.len() {
                let lbl = args[i + 1].as_bytes();
                let len = lbl.len().min(16);
                label[..len].copy_from_slice(&lbl[..len]);
                i += 2;
                continue;
            } else if !b.starts_with(b"-") && target.is_none() {
                target = Some(args[i].clone());
            }
            i += 1;
        }

        let dev_path = match target {
            Some(t) => t,
            None => {
                eprintln!("mkswap: device or file required");
                return Ok(1);
            }
        };

        let mut f = match OpenOptions::new().read(true).write(true).open(&dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("mkswap: {}: {}", dev_path.to_string_lossy(), e);
                return Ok(1);
            }
        };

        let page_size = 4096usize;
        let file_len = match f.metadata() {
            Ok(m) => m.len(),
            Err(e) => {
                eprintln!("mkswap: metadata failed: {}", e);
                return Ok(1);
            }
        };

        if file_len < (page_size * 2) as u64 {
            eprintln!(
                "mkswap: error: swap area needs to be at least {} KiB",
                page_size * 2 / 1024
            );
            return Ok(1);
        }

        let num_pages = (file_len / page_size as u64) as u32;

        let _ = f.write_all(&[0u8; 1024]);

        let mut hdr = [0u8; 1024];
        hdr[0..4].copy_from_slice(&1u32.to_le_bytes());
        hdr[4..8].copy_from_slice(&(num_pages - 1).to_le_bytes());
        hdr[8..12].copy_from_slice(&0u32.to_le_bytes());

        hdr[28..44].copy_from_slice(&label);

        let _ = f.write_all(&hdr);

        let sig_pos = (page_size - 10) as u64;
        let _ = io::Seek::seek(&mut f, io::SeekFrom::Start(sig_pos));
        let _ = f.write_all(b"SWAPSPACE2");
        let _ = f.sync_all();

        println!(
            "Setting up swapspace version 1, size = {} bytes",
            file_len - page_size as u64
        );

        Ok(0)
    }
}

pub struct SwaponApplet;
impl Applet for SwaponApplet {
    fn name(&self) -> &'static str {
        "swapon"
    }
    fn description(&self) -> &'static str {
        "Start swapping on specified device/file"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut flags: libc::c_int = 0;
        let mut targets = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-a" {
                let fstab = fs::read_to_string("/etc/fstab").unwrap_or_default();
                for line in fstab.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 && parts[2] == "swap" {
                        targets.push(OsString::from(parts[0]));
                    }
                }
            } else if b == b"-p" && i + 1 < args.len() {
                if let Ok(prio) = args[i + 1].to_string_lossy().parse::<libc::c_int>() {
                    flags |= 0x8000 | (prio & 0x7fff);
                }
                i += 2;
                continue;
            } else if b == b"-d" {
                flags |= 0x10000;
            } else if !b.starts_with(b"-") {
                targets.push(args[i].clone());
            }
            i += 1;
        }

        if targets.is_empty() {
            eprintln!("swapon: [-a] [DEVICE]");
            return Ok(1);
        }

        let mut status = 0;
        for t in targets {
            let c_path = match CString::new(t.as_bytes()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let res = unsafe { libc::swapon(c_path.as_ptr(), flags) };
            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("swapon: {}: {}", t.to_string_lossy(), err);
                status = 1;
            }
        }

        Ok(status)
    }
}

pub struct SwapoffApplet;
impl Applet for SwapoffApplet {
    fn name(&self) -> &'static str {
        "swapoff"
    }
    fn description(&self) -> &'static str {
        "Stop swapping on specified device/file"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut targets = Vec::new();

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-a" {
                let swaps = fs::read_to_string("/proc/swaps").unwrap_or_default();
                for line in swaps.lines().skip(1) {
                    if let Some(dev) = line.split_whitespace().next() {
                        targets.push(OsString::from(dev));
                    }
                }
            } else if !b.starts_with(b"-") {
                targets.push(arg.clone());
            }
        }

        if targets.is_empty() {
            eprintln!("swapoff: [-a] [DEVICE]");
            return Ok(1);
        }

        let mut status = 0;
        for t in targets {
            let c_path = match CString::new(t.as_bytes()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let res = unsafe { libc::swapoff(c_path.as_ptr()) };
            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("swapoff: {}: {}", t.to_string_lossy(), err);
                status = 1;
            }
        }

        Ok(status)
    }
}

pub struct FsfreezeApplet;
impl Applet for FsfreezeApplet {
    fn name(&self) -> &'static str {
        "fsfreeze"
    }
    fn description(&self) -> &'static str {
        "Flush and halt writes to mount point"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut freeze = None;
        let mut mountpoint = None;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"--freeze" || b == b"-f" {
                freeze = Some(true);
            } else if b == b"--unfreeze" || b == b"-u" {
                freeze = Some(false);
            } else if !b.starts_with(b"-") && mountpoint.is_none() {
                mountpoint = Some(arg.clone());
            }
        }

        let mnt = match (freeze, mountpoint) {
            (Some(f), Some(m)) => (f, m),
            _ => {
                eprintln!("fsfreeze: --freeze|--unfreeze MOUNTPOINT");
                return Ok(1);
            }
        };

        let c_path = match CString::new(mnt.1.as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
        if fd < 0 {
            let err = io::Error::last_os_error();
            eprintln!("fsfreeze: open {}: {}", mnt.1.to_string_lossy(), err);
            return Ok(1);
        }

        let op = if mnt.0 { FIFREEZE } else { FITHAW };
        let res = unsafe { libc::ioctl(fd, op, 0) };
        unsafe { libc::close(fd) };

        if res != 0 {
            let err = io::Error::last_os_error();
            eprintln!("fsfreeze: ioctl: {}", err);
            return Ok(1);
        }

        Ok(0)
    }
}

pub struct TtysizeApplet;
impl Applet for TtysizeApplet {
    fn name(&self) -> &'static str {
        "ttysize"
    }
    fn description(&self) -> &'static str {
        "Print dimensions of terminal or default to 80 24"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut ws = MaybeUninit::<Winsize>::uninit();
        let (mut w, mut h) = (80u16, 24u16);

        for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
            if unsafe { libc::ioctl(fd, TIOCGWINSZ, ws.as_mut_ptr()) } == 0 {
                let ws = unsafe { ws.assume_init() };
                w = ws.ws_col;
                h = ws.ws_row;
                break;
            }
        }

        if args.is_empty() {
            println!("{} {}", w, h);
            return Ok(0);
        }

        let mut out = Vec::new();
        for arg in args {
            let s = arg.to_string_lossy();
            if s.contains('w') {
                out.push(format!("{}", w));
            }
            if s.contains('h') {
                out.push(format!("{}", h));
            }
        }

        println!("{}", out.join(" "));
        Ok(0)
    }
}

pub struct OpenvtApplet;
impl Applet for OpenvtApplet {
    fn name(&self) -> &'static str {
        "openvt"
    }
    fn description(&self) -> &'static str {
        "Start a program on a new virtual terminal"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut vtno: Option<i32> = None;
        let mut switch = false;
        let mut wait = false;
        let mut cmd_idx = args.len();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-c" && i + 1 < args.len() {
                vtno = args[i + 1].to_string_lossy().parse().ok();
                i += 2;
                continue;
            } else if b == b"-s" {
                switch = true;
            } else if b == b"-w" {
                wait = true;
            } else if !b.starts_with(b"-") {
                cmd_idx = i;
                break;
            }
            i += 1;
        }

        let c_console = CString::new("/dev/console").unwrap();
        let console_fd = unsafe { libc::open(c_console.as_ptr(), libc::O_RDWR | libc::O_NONBLOCK) };
        if console_fd < 0 {
            eprintln!("openvt: can't open /dev/console");
            return Ok(1);
        }

        let vt_num = match vtno {
            Some(v) => v,
            None => {
                let mut free_vt = 0i32;
                if unsafe { libc::ioctl(console_fd, VT_OPENQRY, &mut free_vt) } != 0 || free_vt <= 0
                {
                    unsafe { libc::close(console_fd) };
                    eprintln!("openvt: can't find free VT");
                    return Ok(1);
                }
                free_vt
            }
        };

        if switch {
            unsafe {
                libc::ioctl(console_fd, VT_ACTIVATE, vt_num);
                libc::ioctl(console_fd, VT_WAITACTIVE, vt_num);
            }
        }
        unsafe { libc::close(console_fd) };

        let vt_dev = format!("/dev/tty{}", vt_num);
        let vt_c = CString::new(vt_dev).unwrap();
        let vt_fd = unsafe { libc::open(vt_c.as_ptr(), libc::O_RDWR) };

        let cmd_parts = if cmd_idx < args.len() {
            &args[cmd_idx..]
        } else {
            &[]
        };

        let prog = if !cmd_parts.is_empty() {
            cmd_parts[0].to_string_lossy().to_string()
        } else {
            env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into())
        };

        let mut cmd = Command::new(&prog);
        if cmd_parts.len() > 1 {
            cmd.args(&cmd_parts[1..]);
        }

        if vt_fd >= 0 {
            let stdio_fd = unsafe { Stdio::from_raw_fd(vt_fd) };
            cmd.stdin(stdio_fd);
        }

        match cmd.spawn() {
            Ok(mut child) => {
                if wait {
                    let st = child.wait().map(|s| s.code().unwrap_or(0)).unwrap_or(1);
                    Ok(st)
                } else {
                    Ok(0)
                }
            }
            Err(e) => {
                eprintln!("openvt: {}: {}", prog, e);
                Ok(1)
            }
        }
    }
}

pub struct WatchApplet;
impl Applet for WatchApplet {
    fn name(&self) -> &'static str {
        "watch"
    }
    fn description(&self) -> &'static str {
        "Execute a program periodically, showing output fullscreen"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut interval = 2u64;
        let mut cmd_idx = args.len();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-n" && i + 1 < args.len() {
                interval = args[i + 1].to_string_lossy().parse().unwrap_or(2);
                i += 2;
                continue;
            } else if !b.starts_with(b"-") {
                cmd_idx = i;
                break;
            }
            i += 1;
        }

        if cmd_idx >= args.len() {
            eprintln!("watch: [-n SEC] PROG [ARGS]");
            return Ok(1);
        }

        let prog = &args[cmd_idx];
        let prog_args = &args[cmd_idx + 1..];

        loop {
            print!("\x1b[2J\x1b[H");
            println!(
                "Every {}s: {}",
                interval,
                args[cmd_idx..]
                    .iter()
                    .map(|a| a.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            println!();
            let _ = io::stdout().flush();

            let _ = Command::new(prog).args(prog_args).status();

            thread::sleep(Duration::from_secs(interval));
        }
    }
}

pub struct SetlogconsApplet;
impl Applet for SetlogconsApplet {
    fn name(&self) -> &'static str {
        "setlogcons"
    }
    fn description(&self) -> &'static str {
        "Pin kernel output to VT console N"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let vtno: u8 = if !args.is_empty() {
            args[0].to_string_lossy().parse().unwrap_or(0)
        } else {
            0
        };

        #[repr(C)]
        struct TiocLinuxArg {
            fn_: u8,
            subarg: u8,
        }

        let arg = TiocLinuxArg {
            fn_: 11,
            subarg: vtno,
        };

        let tty_path = format!("/dev/tty{}", vtno);
        let c_path = CString::new(tty_path).unwrap();
        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
        let target_fd = if fd >= 0 { fd } else { libc::STDIN_FILENO };

        let res = unsafe { libc::ioctl(target_fd, TIOCLINUX, &arg) };
        if fd >= 0 {
            unsafe { libc::close(fd) };
        }

        if res != 0 {
            let err = io::Error::last_os_error();
            eprintln!("setlogcons: {}", err);
            return Ok(1);
        }

        Ok(0)
    }
}

pub struct SetserialApplet;
impl Applet for SetserialApplet {
    fn name(&self) -> &'static str {
        "setserial"
    }
    fn description(&self) -> &'static str {
        "Retrieve or set Linux serial port"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("setserial: DEVICE [PARAMETERS]");
            return Ok(1);
        }

        let dev = &args[0];
        let c_path = match CString::new(dev.as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDWR | libc::O_NONBLOCK) };
        if fd < 0 {
            let err = io::Error::last_os_error();
            eprintln!("setserial: {}: {}", dev.to_string_lossy(), err);
            return Ok(1);
        }

        let mut serinfo = MaybeUninit::<SerialStruct>::zeroed();
        let res = unsafe { libc::ioctl(fd, TIOCGSERIAL, serinfo.as_mut_ptr()) };
        if res != 0 {
            unsafe { libc::close(fd) };
            let err = io::Error::last_os_error();
            eprintln!("setserial: {}: {}", dev.to_string_lossy(), err);
            return Ok(1);
        }
        let mut serinfo = unsafe { serinfo.assume_init() };

        if args.len() == 1 {
            println!(
                "{}, Line {}, UART: {}, Port: 0x{:x}, IRQ: {}",
                dev.to_string_lossy(),
                serinfo.line,
                serinfo.type_,
                serinfo.port,
                serinfo.irq
            );
            unsafe { libc::close(fd) };
            return Ok(0);
        }

        let mut i = 1;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "port" && i + 1 < args.len() {
                let p_str = args[i + 1].to_string_lossy();
                if let Some(hex) = p_str.strip_prefix("0x") {
                    serinfo.port = u32::from_str_radix(hex, 16).unwrap_or(serinfo.port);
                } else {
                    serinfo.port = p_str.parse().unwrap_or(serinfo.port);
                }
                i += 2;
                continue;
            } else if s == "irq" && i + 1 < args.len() {
                serinfo.irq = args[i + 1].to_string_lossy().parse().unwrap_or(serinfo.irq);
                i += 2;
                continue;
            } else if s == "baud_base" && i + 1 < args.len() {
                serinfo.baud_base = args[i + 1]
                    .to_string_lossy()
                    .parse()
                    .unwrap_or(serinfo.baud_base);
                i += 2;
                continue;
            }
            i += 1;
        }

        let set_res = unsafe { libc::ioctl(fd, TIOCSSERIAL, &serinfo) };
        unsafe { libc::close(fd) };

        if set_res != 0 {
            let err = io::Error::last_os_error();
            eprintln!("setserial: cannot set parameters: {}", err);
            return Ok(1);
        }

        Ok(0)
    }
}

pub struct PartprobeApplet;
impl Applet for PartprobeApplet {
    fn name(&self) -> &'static str {
        "partprobe"
    }
    fn description(&self) -> &'static str {
        "Inform OS kernel of partition table changes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut devs = Vec::new();
        for arg in args {
            if !arg.as_bytes().starts_with(b"-") {
                devs.push(arg.clone());
            }
        }

        if devs.is_empty() {
            return Ok(0);
        }

        let mut status = 0;
        for dev in devs {
            let c_path = match CString::new(dev.as_bytes()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
            if fd < 0 {
                let err = io::Error::last_os_error();
                eprintln!("partprobe: {}: {}", dev.to_string_lossy(), err);
                status = 1;
                continue;
            }

            let res = unsafe { libc::ioctl(fd, BLKRRPART, 0) };
            unsafe { libc::close(fd) };

            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("partprobe: {}: {}", dev.to_string_lossy(), err);
                status = 1;
            }
        }

        Ok(status)
    }
}

pub struct NbdClientApplet;
impl Applet for NbdClientApplet {
    fn name(&self) -> &'static str {
        "nbd-client"
    }
    fn description(&self) -> &'static str {
        "Network block device client"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut disconnect = false;
        let mut block_size = 4096u32;
        let mut timeout = 0u32;
        let mut positional = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-d" {
                disconnect = true;
            } else if (b == b"-b" || b == b"-block-size") && i + 1 < args.len() {
                block_size = args[i + 1].to_string_lossy().parse().unwrap_or(4096);
                i += 2;
                continue;
            } else if (b == b"-t" || b == b"-timeout") && i + 1 < args.len() {
                timeout = args[i + 1].to_string_lossy().parse().unwrap_or(0);
                i += 2;
                continue;
            } else if !b.starts_with(b"-") {
                positional.push(args[i].clone());
            }
            i += 1;
        }

        if disconnect {
            if positional.is_empty() {
                eprintln!("nbd-client: -d BLOCKDEV");
                return Ok(1);
            }
            let c_dev = CString::new(positional[0].as_bytes()).unwrap();
            let nbd_fd = unsafe { libc::open(c_dev.as_ptr(), libc::O_RDWR) };
            if nbd_fd < 0 {
                eprintln!(
                    "nbd-client: cannot open {}",
                    positional[0].to_string_lossy()
                );
                return Ok(1);
            }
            unsafe {
                libc::ioctl(nbd_fd, NBD_DISCONNECT, 0);
                libc::ioctl(nbd_fd, NBD_CLEAR_SOCK, 0);
                libc::close(nbd_fd);
            }
            return Ok(0);
        }

        if positional.len() < 2 {
            eprintln!("nbd-client: HOST [PORT] BLOCKDEV");
            return Ok(1);
        }

        let host = positional[0].to_string_lossy().to_string();
        let (port, device) = if positional.len() >= 3 {
            (
                positional[1]
                    .to_string_lossy()
                    .parse::<u16>()
                    .unwrap_or(10809),
                positional[2].clone(),
            )
        } else {
            (10809, positional[1].clone())
        };

        let c_dev = CString::new(device.as_bytes()).unwrap();
        let nbd_fd = unsafe { libc::open(c_dev.as_ptr(), libc::O_RDWR) };
        if nbd_fd < 0 {
            eprintln!("nbd-client: open device: {}", io::Error::last_os_error());
            return Ok(1);
        }

        let stream = match TcpStream::connect((host.as_str(), port)) {
            Ok(s) => s,
            Err(e) => {
                unsafe { libc::close(nbd_fd) };
                eprintln!("nbd-client: connect to {}:{}: {}", host, port, e);
                return Ok(1);
            }
        };

        let sock_fd = stream.as_raw_fd();

        unsafe {
            if timeout > 0 {
                libc::ioctl(nbd_fd, NBD_SET_TIMEOUT, timeout as libc::c_ulong);
            }
            libc::ioctl(nbd_fd, NBD_SET_BLKSIZE, block_size as libc::c_ulong);
            libc::ioctl(nbd_fd, NBD_SET_SOCK, sock_fd);
            libc::ioctl(nbd_fd, NBD_DO_IT, 0);
            libc::ioctl(nbd_fd, NBD_CLEAR_QUE, 0);
            libc::ioctl(nbd_fd, NBD_CLEAR_SOCK, 0);
            libc::close(nbd_fd);
        }

        Ok(0)
    }
}
