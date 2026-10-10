use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::fs::{self};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct ResumeApplet;

impl Applet for ResumeApplet {
    fn name(&self) -> &'static str {
        "resume"
    }
    fn description(&self) -> &'static str {
        "Resume from software suspend"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let resume_path = Path::new("/sys/power/resume");
        if args.is_empty() {
            match fs::read_to_string(resume_path) {
                Ok(s) => {
                    print_bytes(s.as_bytes());
                    Ok(0)
                }
                Err(e) => {
                    eprintln!("resume: cannot read {}: {}", resume_path.display(), e);
                    Ok(1)
                }
            }
        } else {
            let dev_spec = args[0].as_bytes();

            let dev_str = if dev_spec.starts_with(b"/dev/") {
                let path = Path::new(&args[0]);
                match fs::metadata(path) {
                    Ok(meta) => {
                        use std::os::unix::fs::MetadataExt;
                        let rdev = meta.rdev();
                        let maj = libc::major(rdev);
                        let min = libc::minor(rdev);
                        format!("{}:{}\n", maj, min)
                    }
                    Err(e) => {
                        eprintln!("resume: stat {}: {}", path.display(), e);
                        return Ok(1);
                    }
                }
            } else {
                let mut s = String::from_utf8_lossy(dev_spec).into_owned();
                s.push('\n');
                s
            };

            if let Err(e) = fs::write(resume_path, dev_str.as_bytes()) {
                eprintln!("resume: cannot write {}: {}", resume_path.display(), e);
                return Ok(1);
            }
            Ok(0)
        }
    }
}
