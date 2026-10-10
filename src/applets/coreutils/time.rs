use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

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
        let mut format_str: Option<&str> = None;

        while idx < args.len() {
            let b = args[idx].as_bytes();
            if b == b"-v" || b == b"--verbose" {
                verbose = true;
            } else if b == b"-p" || b == b"--portability" {
                // Posix portability mode
            } else if b == b"-f" || b == b"--format" {
                idx += 1;
                if idx < args.len() {
                    format_str = args[idx].to_str();
                }
            } else if b.starts_with(b"-f") && b.len() > 2 {
                format_str = std::str::from_utf8(&b[2..]).ok();
            } else if b.starts_with(b"-") && b != b"--" {
            } else {
                if b == b"--" {
                    idx += 1;
                }
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
        let elapsed_ms = elapsed.as_millis() as u64;
        let user_secs = (ru_after.ru_utime.tv_sec - ru_before.ru_utime.tv_sec) as f64
            + (ru_after.ru_utime.tv_usec - ru_before.ru_utime.tv_usec) as f64 / 1_000_000.0;
        let sys_secs = (ru_after.ru_stime.tv_sec - ru_before.ru_stime.tv_sec) as f64
            + (ru_after.ru_stime.tv_usec - ru_before.ru_stime.tv_usec) as f64 / 1_000_000.0;

        let exit_code = match &status {
            Ok(st) => st.code().unwrap_or(1),
            Err(_) => 127,
        };

        if let Some(fmt) = format_str {
            summarize(fmt, cmd.to_string_lossy().as_ref(), elapsed_ms, user_secs, sys_secs, exit_code);
        } else if verbose {
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

fn summarize(
    fmt: &str,
    cmd_name: &str,
    elapsed_ms: u64,
    user_secs: f64,
    sys_secs: f64,
    exit_code: i32,
) {
    let bytes = fmt.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    let mut trailing_percent_suppress_nl = false;

    while i < bytes.len() {
        if bytes[i] == b'%' {
            i += 1;
            if i >= bytes.len() {
                out.push(b'?');
                trailing_percent_suppress_nl = true;
                break;
            }
            match bytes[i] {
                b'%' => out.push(b'%'),
                b'C' => out.extend_from_slice(cmd_name.as_bytes()),
                b'E' => {
                    let seconds = elapsed_ms / 1000;
                    if seconds >= 3600 {
                        out.extend_from_slice(
                            format!("{}h {}m {:02}s", seconds / 3600, (seconds % 3600) / 60, seconds % 60)
                                .as_bytes(),
                        );
                    } else {
                        out.extend_from_slice(
                            format!("{}m {}.{:02}s", seconds / 60, seconds % 60, (elapsed_ms / 10) % 100)
                                .as_bytes(),
                        );
                    }
                }
                b'e' => {
                    out.extend_from_slice(
                        format!("{}.{:02}", elapsed_ms / 1000, (elapsed_ms / 10) % 100).as_bytes(),
                    );
                }
                b'U' => {
                    out.extend_from_slice(format!("{:.2}", user_secs).as_bytes());
                }
                b'S' => {
                    out.extend_from_slice(format!("{:.2}", sys_secs).as_bytes());
                }
                b'x' => {
                    out.extend_from_slice(format!("{}", exit_code).as_bytes());
                }
                c => {
                    out.push(b'?');
                    out.push(c);
                }
            }
        } else if bytes[i] == b'\\' {
            i += 1;
            if i >= bytes.len() {
                out.extend_from_slice(b"?\\");
                out.extend_from_slice(cmd_name.as_bytes());
                break;
            }
            match bytes[i] {
                b'\\' => out.push(b'\\'),
                b't' => out.push(b'\t'),
                b'n' => out.push(b'\n'),
                c => {
                    out.extend_from_slice(b"?\\");
                    out.push(c);
                }
            }
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }

    if !trailing_percent_suppress_nl {
        out.push(b'\n');
    }

    use std::io::Write;
    let _ = std::io::stderr().write_all(&out);
}
