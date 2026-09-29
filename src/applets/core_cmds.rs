use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::thread;
use std::time::Duration;
use crate::core::{Applet, Result};

pub struct TrueApplet;
impl Applet for TrueApplet {
    fn name(&self) -> &'static str { "true" }
    fn description(&self) -> &'static str { "Return an exit code of success" }
    fn run(&self, _args: &[OsString]) -> Result<i32> { Ok(0) }
}

pub struct FalseApplet;
impl Applet for FalseApplet {
    fn name(&self) -> &'static str { "false" }
    fn description(&self) -> &'static str { "Return an exit code of a failure" }
    fn run(&self, _args: &[OsString]) -> Result<i32> { Ok(1) }
}

pub struct PwdApplet;
impl Applet for PwdApplet {
    fn name(&self) -> &'static str { "pwd" }
    fn description(&self) -> &'static str { "Print the current working directory" }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let dir = env::current_dir()?;
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(dir.as_os_str().as_bytes())?;
        handle.write_all(b"\n")?;
        Ok(0)
    }
}

pub struct EchoApplet;
impl Applet for EchoApplet {
    fn name(&self) -> &'static str { "echo" }
    fn description(&self) -> &'static str { "Display a line of text" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut no_newline = false;
        let mut interpret_escapes = false;
        let mut idx = 0;

        while idx < args.len() {
            let arg = &args[idx];
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 {
                let rest = &bytes[1..];
                if rest.iter().all(|&b| b == b'n' || b == b'e' || b == b'E') {
                    for &b in rest {
                        match b {
                            b'n' => no_newline = true,
                            b'e' => interpret_escapes = true,
                            b'E' => interpret_escapes = false,
                            _ => {}
                        }
                    }
                    idx += 1;
                    continue;
                }
            }
            break;
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let mut first = true;

        for arg in &args[idx..] {
            if !first {
                handle.write_all(b" ")?;
            }
            first = false;

            let bytes = arg.as_bytes();
            if interpret_escapes {
                let mut i = 0;
                let mut stop = false;
                while i < bytes.len() {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 1;
                        match bytes[i] {
                            b'a' => { handle.write_all(b"\x07")?; }
                            b'b' => { handle.write_all(b"\x08")?; }
                            b'c' => { stop = true; break; }
                            b'e' | b'E' => { handle.write_all(b"\x1b")?; }
                            b'f' => { handle.write_all(b"\x0c")?; }
                            b'n' => { handle.write_all(b"\n")?; }
                            b'r' => { handle.write_all(b"\r")?; }
                            b't' => { handle.write_all(b"\t")?; }
                            b'v' => { handle.write_all(b"\x0b")?; }
                            b'\\' => { handle.write_all(b"\\")?; }
                            b'0'..=b'7' => {
                                let mut val: u32 = (bytes[i] - b'0') as u32;
                                let mut count = 1;
                                if val == 0 {
                                    while count < 4 && i + 1 < bytes.len() && bytes[i + 1] >= b'0' && bytes[i + 1] <= b'7' {
                                        i += 1;
                                        val = (val << 3) + (bytes[i] - b'0') as u32;
                                        count += 1;
                                    }
                                } else {
                                    while count < 3 && i + 1 < bytes.len() && bytes[i + 1] >= b'0' && bytes[i + 1] <= b'7' {
                                        i += 1;
                                        val = (val << 3) + (bytes[i] - b'0') as u32;
                                        count += 1;
                                    }
                                }
                                handle.write_all(&[(val & 0xFF) as u8])?;
                            }
                            b'x' => {
                                let mut val = 0u32;
                                let mut found = false;
                                while i + 1 < bytes.len() && (bytes[i + 1] as char).is_ascii_hexdigit() {
                                    i += 1;
                                    val = (val << 4) + match bytes[i] {
                                        b'0'..=b'9' => (bytes[i] - b'0') as u32,
                                        b'a'..=b'f' => (bytes[i] - b'a' + 10) as u32,
                                        b'A'..=b'F' => (bytes[i] - b'A' + 10) as u32,
                                        _ => 0,
                                    };
                                    found = true;
                                }
                                if found {
                                    handle.write_all(&[(val & 0xFF) as u8])?;
                                } else {
                                    handle.write_all(b"\\x")?;
                                }
                            }
                            other => {
                                handle.write_all(&[b'\\', other])?;
                            }
                        }
                    } else {
                        handle.write_all(&[bytes[i]])?;
                    }
                    i += 1;
                }
                if stop {
                    return Ok(0);
                }
            } else {
                handle.write_all(bytes)?;
            }
        }

        if !no_newline {
            handle.write_all(b"\n")?;
        }
        handle.flush()?;
        Ok(0)
    }
}

pub struct PrintenvApplet;
impl Applet for PrintenvApplet {
    fn name(&self) -> &'static str { "printenv" }
    fn description(&self) -> &'static str { "Print all or part of environment" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        if args.is_empty() {
            for (k, v) in env::vars_os() {
                handle.write_all(k.as_bytes())?;
                handle.write_all(b"=")?;
                handle.write_all(v.as_bytes())?;
                handle.write_all(b"\n")?;
            }
            Ok(0)
        } else {
            let mut ret = 0;
            for arg in args {
                match env::var_os(arg) {
                    Some(val) => {
                        handle.write_all(val.as_bytes())?;
                        handle.write_all(b"\n")?;
                    }
                    None => ret = 1,
                }
            }
            Ok(ret)
        }
    }
}

pub struct SleepApplet;
impl Applet for SleepApplet {
    fn name(&self) -> &'static str { "sleep" }
    fn description(&self) -> &'static str { "Delay for a specified amount of time" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("sleep: missing operand");
            return Ok(1);
        }
        let mut total_secs: f64 = 0.0;
        for arg in args {
            let s = arg.to_string_lossy();
            let (val_str, unit) = if s.ends_with('s') {
                (&s[..s.len()-1], 1.0)
            } else if s.ends_with('m') {
                (&s[..s.len()-1], 60.0)
            } else if s.ends_with('h') {
                (&s[..s.len()-1], 3600.0)
            } else if s.ends_with('d') {
                (&s[..s.len()-1], 86400.0)
            } else {
                (s.as_ref(), 1.0)
            };
            match val_str.parse::<f64>() {
                Ok(v) => total_secs += v * unit,
                Err(_) => {
                    eprintln!("sleep: invalid number '{}'", s);
                    return Ok(1);
                }
            }
        }
        thread::sleep(Duration::from_secs_f64(total_secs));
        Ok(0)
    }
}

pub struct YesApplet;
impl Applet for YesApplet {
    fn name(&self) -> &'static str { "yes" }
    fn description(&self) -> &'static str { "Output a string repeatedly until killed" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let msg = if args.is_empty() {
            b"y".to_vec()
        } else {
            let mut out = Vec::new();
            for (i, a) in args.iter().enumerate() {
                if i > 0 { out.push(b' '); }
                out.extend_from_slice(a.as_bytes());
            }
            out
        };
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        loop {
            if handle.write_all(&msg).is_err() || handle.write_all(b"\n").is_err() {
                break;
            }
        }
        Ok(0)
    }
}

pub struct WhoamiApplet;
impl Applet for WhoamiApplet {
    fn name(&self) -> &'static str { "whoami" }
    fn description(&self) -> &'static str { "Print effective user name" }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        if let Some(user) = crate::core::platform::get_current_username() {
            println!("{}", user);
            Ok(0)
        } else {
            eprintln!("whoami: cannot find name for user ID");
            Ok(1)
        }
    }
}

pub struct ArchApplet;
impl Applet for ArchApplet {
    fn name(&self) -> &'static str { "arch" }
    fn description(&self) -> &'static str { "Print machine architecture" }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        println!("{}", crate::core::platform::get_machine_arch());
        Ok(0)
    }
}

pub struct NprocApplet;
impl Applet for NprocApplet {
    fn name(&self) -> &'static str { "nproc" }
    fn description(&self) -> &'static str { "Print the number of processing units available" }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let cpus = thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        println!("{}", cpus);
        Ok(0)
    }
}

pub struct SyncApplet;
impl Applet for SyncApplet {
    fn name(&self) -> &'static str { "sync" }
    fn description(&self) -> &'static str { "Force changed blocks to disk" }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        crate::core::platform::sync_disks();
        Ok(0)
    }
}

pub struct ClearApplet;
impl Applet for ClearApplet {
    fn name(&self) -> &'static str { "clear" }
    fn description(&self) -> &'static str { "Clear the terminal screen" }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        print!("\x1b[H\x1b[2J\x1b[3J");
        let _ = io::stdout().flush();
        Ok(0)
    }
}

pub struct ResetApplet;
impl Applet for ResetApplet {
    fn name(&self) -> &'static str { "reset" }
    fn description(&self) -> &'static str { "Reset the terminal" }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        print!("\x1bc");
        let _ = io::stdout().flush();
        Ok(0)
    }
}
