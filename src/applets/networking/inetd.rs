use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub struct InetdApplet;

impl Applet for InetdApplet {
    fn name(&self) -> &'static str {
        "inetd"
    }
    fn description(&self) -> &'static str {
        "Internet super-server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let conf_file = if !args.is_empty() && !args[0].as_bytes().starts_with(b"-") {
            args[0].as_os_str()
        } else {
            Path::new("/etc/inetd.conf").as_os_str()
        };

        let file = match File::open(conf_file) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("inetd: {}: {}", conf_file.to_string_lossy(), e);
                return Ok(1);
            }
        };

        let reader = BufReader::new(file);
        for line in reader.lines().map_while(std::result::Result::ok) {
            let l = line.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = l.split_whitespace().collect();

            if parts.len() < 6 {
                continue;
            }
            let service = parts[0];
            let proto = parts[2];
            let prog = parts[5].to_string();
            let prog_args: Vec<String> = if parts.len() > 6 {
                parts[6..].iter().map(|s| s.to_string()).collect()
            } else {
                vec![prog.clone()]
            };

            let port: u16 = service.parse().unwrap_or(0);
            if proto == "tcp" && port > 0 {
                let prog_clone = prog.clone();
                let prog_args_clone = prog_args.clone();
                std::thread::spawn(move || {
                    if let Ok(listener) = TcpListener::bind(("0.0.0.0", port)) {
                        for stream in listener.incoming().flatten() {
                            let fd = stream.as_raw_fd();
                            let stdin_fd = unsafe { libc::dup(fd) };
                            let stdout_fd = unsafe { libc::dup(fd) };
                            if stdin_fd >= 0 && stdout_fd >= 0 {
                                let in_file = unsafe { File::from_raw_fd(stdin_fd) };
                                let out_file = unsafe { File::from_raw_fd(stdout_fd) };
                                let _ = Command::new(&prog_clone)
                                    .args(&prog_args_clone[1..])
                                    .stdin(Stdio::from(in_file))
                                    .stdout(Stdio::from(out_file))
                                    .spawn();
                            }
                        }
                    }
                });
            }
        }

        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    }
}
