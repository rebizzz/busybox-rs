use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

fn supervise_pid(dir: &Path) -> Option<i32> {
    let data = std::fs::read(dir.join("supervise").join("pid")).ok()?;
    String::from_utf8_lossy(&data).trim().parse().ok()
}

fn supervise_stat(dir: &Path) -> Option<String> {
    let data = std::fs::read(dir.join("supervise").join("stat")).ok()?;
    Some(String::from_utf8_lossy(&data).trim().to_owned())
}

fn pid_alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

fn sig_send(pid: i32, sig: i32) -> bool {
    unsafe { libc::kill(pid, sig) == 0 }
}

pub struct RunsvApplet;
impl Applet for RunsvApplet {
    fn name(&self) -> &'static str {
        "runsv"
    }
    fn description(&self) -> &'static str {
        "Supervise single service dir: run ./run, restart on exit"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let dir = if args.is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(&args[0])
        };
        let run = dir.join("run");
        let finish = dir.join("finish");
        let _ = std::fs::create_dir_all(dir.join("supervise"));
        loop {
            if dir.join("down").exists() {
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
            let mut child = match Command::new(&run).current_dir(&dir).spawn() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("runsv: {}: {e}", run.display());
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    continue;
                }
            };
            let pid = child.id() as i32;
            let _ = std::fs::write(dir.join("supervise").join("pid"), pid.to_string());
            let _ = std::fs::write(dir.join("supervise").join("stat"), "run");
            let status = child.wait();
            let code = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
            if finish.exists() {
                let _ = Command::new(&finish)
                    .current_dir(&dir)
                    .arg(code.to_string())
                    .status();
            }
            let _ = std::fs::write(dir.join("supervise").join("stat"), "down");
            eprintln!("runsv: {pid} exited {code}, restarting");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
