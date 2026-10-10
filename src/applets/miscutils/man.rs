use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ManApplet;
impl Applet for ManApplet {
    fn name(&self) -> &'static str {
        "man"
    }
    fn description(&self) -> &'static str {
        "Format and display the on-line manual pages"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let topic = match args.iter().find(|a| !a.as_bytes().starts_with(b"-")) {
            Some(t) => t.to_string_lossy(),
            None => {
                eprintln!("What manual page do you want?");
                return Ok(1);
            }
        };

        let man_dirs = [
            "/usr/share/man",
            "/usr/local/share/man",
            "/usr/man",
            "/nix/var/nix/profiles/default/share/man",
        ];
        let sections = ["man1", "man8", "man5", "man7", "man2", "man3"];

        let mut target_file: Option<PathBuf> = None;
        for dir in &man_dirs {
            for sec in &sections {
                let p = Path::new(dir).join(sec).join(format!("{}.1", topic));
                if p.exists() {
                    target_file = Some(p);
                    break;
                }
                let p_gz = Path::new(dir).join(sec).join(format!("{}.1.gz", topic));
                if p_gz.exists() {
                    target_file = Some(p_gz);
                    break;
                }
            }
            if target_file.is_some() {
                break;
            }
        }

        let file_path = match target_file {
            Some(p) => p,
            None => {
                eprintln!("No manual entry for {}", topic);
                return Ok(1);
            }
        };

        let content = if file_path.extension().and_then(|e| e.to_str()) == Some("gz") {
            let out = Command::new("gzip")
                .arg("-dc")
                .arg(&file_path)
                .output()
                .map(|o| o.stdout)
                .unwrap_or_default();
            String::from_utf8_lossy(&out).to_string()
        } else {
            fs::read_to_string(&file_path).unwrap_or_default()
        };

        let mut stdout = io::stdout().lock();
        let _ = stdout.write_all(content.as_bytes());
        let _ = stdout.flush();

        Ok(0)
    }
}
