use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

fn sv_status_one(dir: &OsString) -> i32 {
    let p = Path::new(dir);
    let stat = supervise_stat(p).unwrap_or_else(|| "down".to_owned());
    match supervise_pid(p) {
        Some(pid) if pid_alive(pid) => {
            println!("run: {}: (pid {pid})", dir.to_string_lossy());
            let _ = stat;
            0
        }
        _ => {
            println!("down: {}: 0s", dir.to_string_lossy());
            0
        }
    }
}

pub struct SvApplet;
impl Applet for SvApplet {
    fn name(&self) -> &'static str {
        "sv"
    }
    fn description(&self) -> &'static str {
        "Service control via supervise/ dir (status/up/down/once/signals subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut cmd = "status";
        let mut dirs: Vec<&OsString> = Vec::new();
        for a in args {
            let b = ab(a);
            if b == b"-v" {
                continue;
            }
            if !b.starts_with(b"-") || b == b"-1" || b == b"-2" {
                match (cmd, b) {
                    (
                        "status",
                        b"status" | b"up" | b"down" | b"once" | b"pause" | b"cont" | b"hup"
                        | b"alarm" | b"interrupt" | b"quit" | b"term" | b"kill" | b"1" | b"2"
                        | b"-1" | b"-2" | b"start" | b"stop" | b"restart" | b"shutdown"
                        | b"force-stop" | b"force-restart",
                    ) => {
                        cmd = match b {
                            b"status" => "status",
                            b"up" | b"start" => "up",
                            b"down" | b"stop" => "down",
                            b"once" => "once",
                            b"pause" => "pause",
                            b"cont" => "cont",
                            b"hup" => "hup",
                            b"alarm" => "alarm",
                            b"interrupt" => "interrupt",
                            b"quit" => "quit",
                            b"term" => "term",
                            b"kill" => "kill",
                            b"1" => "1",
                            b"2" => "2",
                            b"-1" => "1",
                            b"-2" => "2",
                            _ => "restart",
                        };
                    }
                    _ => dirs.push(a),
                }
            } else {
                eprintln!("sv: unknown option {}", b.escape_ascii());
                return Ok(1);
            }
        }
        if dirs.is_empty() {
            eprintln!("sv: missing service dir");
            return Ok(1);
        }
        let mut rc = 0;
        for d in dirs {
            let p = Path::new(d);
            match cmd {
                "status" => {
                    sv_status_one(d);
                }
                "up" => {
                    let _ = std::fs::remove_file(p.join("down"));
                    match supervise_pid(p) {
                        Some(pid) if pid_alive(pid) => {
                            sig_send(pid, libc::SIGCONT);
                        }
                        _ => eprintln!(
                            "sv: {}: no supervised pid (runsv will start it)",
                            d.to_string_lossy()
                        ),
                    }
                }
                "down" => {
                    let _ = std::fs::write(p.join("down"), "");
                    match supervise_pid(p) {
                        Some(pid) => {
                            if !sig_send(pid, libc::SIGTERM) {
                                eprintln!("sv: {}: signal failed", d.to_string_lossy());
                                rc = 1;
                            }
                        }
                        None => {
                            eprintln!("sv: {}: no supervised pid", d.to_string_lossy());
                            rc = 1;
                        }
                    }
                }
                "once" => {
                    let _ = std::fs::remove_file(p.join("down"));
                    if supervise_pid(p).is_none() {
                        let run = p.join("run");
                        match Command::new(&run).current_dir(p).spawn() {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("sv: {}: {e}", d.to_string_lossy());
                                rc = 1;
                            }
                        }
                    }
                }
                other => {
                    let sig = match other {
                        "pause" => libc::SIGSTOP,
                        "cont" => libc::SIGCONT,
                        "hup" => libc::SIGHUP,
                        "alarm" => libc::SIGALRM,
                        "interrupt" => libc::SIGINT,
                        "quit" => libc::SIGQUIT,
                        "term" | "restart" | "shutdown" | "force-stop" | "force-restart" => {
                            libc::SIGTERM
                        }
                        "kill" => libc::SIGKILL,
                        "1" => libc::SIGUSR1,
                        "2" => libc::SIGUSR2,
                        _ => 0,
                    };
                    match supervise_pid(p) {
                        Some(pid) => {
                            if other.starts_with("restart")
                                || other == "shutdown"
                                || other.starts_with("force")
                            {
                                let _ = std::fs::remove_file(p.join("down"));
                            }
                            if sig != 0 && !sig_send(pid, sig) {
                                eprintln!("sv: {}: signal failed", d.to_string_lossy());
                                rc = 1;
                            }
                        }
                        None => {
                            eprintln!("sv: {}: no supervised pid", d.to_string_lossy());
                            rc = 1;
                        }
                    }
                }
            }
        }
        Ok(rc)
    }
}
