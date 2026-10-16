use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::Command;

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

