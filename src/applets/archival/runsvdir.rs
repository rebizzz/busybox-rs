use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

pub struct RunsvdirApplet;
impl Applet for RunsvdirApplet {
    fn name(&self) -> &'static str {
        "runsvdir"
    }
    fn description(&self) -> &'static str {
        "Scan dir for services, keep a runsv child on each"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let dir = if args.is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(&args[0])
        };
        let me = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("runsv"));
        let mut kids: Vec<(String, std::process::Child)> = Vec::new();
        loop {
            let mut seen: Vec<String> = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for ent in entries.flatten() {
                    let p = ent.path();
                    if !p.is_dir() || !p.join("run").exists() {
                        continue;
                    }
                    let name = ent.file_name().to_string_lossy().into_owned();
                    seen.push(name.clone());
                    if kids.iter().any(|(n, _)| *n == name) {
                        continue;
                    }

                    let mut child = Command::new(&me).arg("runsv").arg(&p).spawn();
                    if child.is_err() {
                        child = Command::new("runsv").arg(&p).spawn();
                    }
                    match child {
                        Ok(c) => kids.push((name, c)),
                        Err(e) => eprintln!("runsvdir: {name}: {e}"),
                    }
                }
            }

            kids.retain_mut(|(n, c)| {
                if !seen.contains(n) {
                    let _ = c.kill();
                    let _ = c.wait();
                    return false;
                }
                match c.try_wait() {
                    Ok(Some(_)) => false,
                    Ok(None) => true,
                    Err(_) => false,
                }
            });
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
