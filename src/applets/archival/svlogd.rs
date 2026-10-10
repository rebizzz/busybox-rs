use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

pub struct SvlogdApplet;
impl Applet for SvlogdApplet {
    fn name(&self) -> &'static str {
        "svlogd"
    }
    fn description(&self) -> &'static str {
        "Log stdin lines to dir/current with size rotation (-s/-n/-tt subset)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut max_size: u64 = 100_000;
        let mut keep: usize = 10;
        let mut tstamp = false;
        let mut dir: Option<PathBuf> = None;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-tt" || b == b"-t" {
                tstamp = true;
            } else if b.starts_with(b"-s") {
                let v = if b.len() > 2 {
                    String::from_utf8_lossy(&b[2..]).into_owned()
                } else if i + 1 < args.len() {
                    i += 1;
                    args[i].to_string_lossy().into_owned()
                } else {
                    String::new()
                };
                max_size = v.parse().unwrap_or(max_size);
            } else if b.starts_with(b"-n") {
                let v = if b.len() > 2 {
                    String::from_utf8_lossy(&b[2..]).into_owned()
                } else if i + 1 < args.len() {
                    i += 1;
                    args[i].to_string_lossy().into_owned()
                } else {
                    String::new()
                };
                keep = v.parse().unwrap_or(keep);
            } else if !b.starts_with(b"-") {
                dir = Some(PathBuf::from(&args[i]));
            }
            i += 1;
        }
        let dir = match dir {
            Some(d) => d,
            None => {
                eprintln!("svlogd: missing log dir");
                return Ok(1);
            }
        };
        if std::fs::create_dir_all(&dir).is_err() {
            eprintln!("svlogd: cannot create {}", dir.display());
            return Ok(1);
        }
        let cur = dir.join("current");
        let mut size = std::fs::metadata(&cur).map(|m| m.len()).unwrap_or(0);
        let stdin = std::io::stdin();
        let lines = std::io::BufRead::lines(stdin.lock());
        let mut f = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&cur)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("svlogd: {e}");
                return Ok(1);
            }
        };
        for line in lines {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let entry = if tstamp {
                format!("{:?} {line}\n", std::time::SystemTime::now())
            } else {
                format!("{line}\n")
            };
            if f.write_all(entry.as_bytes()).is_err() {
                return Ok(1);
            }
            size += entry.len() as u64;
            if size >= max_size {
                drop(f);

                let oldest = dir.join(format!("current.{keep}"));
                let _ = std::fs::remove_file(&oldest);
                let mut k = keep;
                while k > 1 {
                    let _ = std::fs::rename(
                        dir.join(format!("current.{}", k - 1)),
                        dir.join(format!("current.{k}")),
                    );
                    k -= 1;
                }
                let _ = std::fs::rename(&cur, dir.join("current.1"));
                size = 0;
                f = match std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&cur)
                {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("svlogd: {e}");
                        return Ok(1);
                    }
                };
            }
        }
        Ok(0)
    }
}
