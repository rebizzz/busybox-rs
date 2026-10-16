use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::path::Path;

pub struct SvcApplet;
impl Applet for SvcApplet {
    fn name(&self) -> &'static str {
        "svc"
    }
    fn description(&self) -> &'static str {
        "Send service commands (-u/-d/-o/-p/-c/-h/-t/-k/-x subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut flags: Vec<u8> = Vec::new();
        let mut dirs: Vec<&OsString> = Vec::new();
        for a in args {
            let b = ab(a);
            if b.starts_with(b"-") && b.len() > 1 {
                flags.extend_from_slice(&b[1..]);
            } else {
                dirs.push(a);
            }
        }
        if flags.is_empty() || dirs.is_empty() {
            eprintln!("svc: usage: svc -u/-d/... servicedir...");
            return Ok(1);
        }
        let mut rc = 0;
        for d in dirs {
            let p = Path::new(d);
            for f in &flags {
                match f {
                    b'u' => {
                        let _ = std::fs::remove_file(p.join("down"));
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGCONT);
                        }
                    }
                    b'd' => {
                        let _ = std::fs::write(p.join("down"), "");
                        match supervise_pid(p) {
                            Some(pid) => {
                                sig_send(pid, libc::SIGTERM);
                            }
                            None => {
                                eprintln!("svc: {}: no supervised pid", d.to_string_lossy());
                                rc = 1;
                            }
                        }
                    }
                    b'o' => {
                        let _ = std::fs::remove_file(p.join("down"));
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGCONT);
                        }
                    }
                    b'p' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGSTOP);
                        } else {
                            rc = 1;
                        }
                    }
                    b'c' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGCONT);
                        } else {
                            rc = 1;
                        }
                    }
                    b'h' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGHUP);
                        } else {
                            rc = 1;
                        }
                    }
                    b'a' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGALRM);
                        } else {
                            rc = 1;
                        }
                    }
                    b'i' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGINT);
                        } else {
                            rc = 1;
                        }
                    }
                    b't' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGTERM);
                        } else {
                            rc = 1;
                        }
                    }
                    b'k' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGKILL);
                        } else {
                            rc = 1;
                        }
                    }
                    b'x' | b'X' => {
                        if let Some(pid) = supervise_pid(p) {
                            sig_send(pid, libc::SIGTERM);
                        } else {
                            rc = 1;
                        }
                    }
                    _ => {
                        eprintln!("svc: unknown flag -{}", *f as char);
                        rc = 1;
                    }
                }
            }
        }
        Ok(rc)
    }
}
