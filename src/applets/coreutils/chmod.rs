use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::fs::{self};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct ChmodApplet;
impl Applet for ChmodApplet {
    fn name(&self) -> &'static str {
        "chmod"
    }
    fn description(&self) -> &'static str {
        "Change file mode bits"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut recursive = false;
        let mut pos: Vec<&OsString> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let a = &args[i];
            let b = a.as_bytes();
            if b == b"--" {
                for r in &args[i + 1..] {
                    pos.push(r);
                }
                break;
            } else if b.starts_with(b"-") && b.len() > 1 {
                for &c in &b[1..] {
                    if c == b'R' {
                        recursive = true;
                    } else {
                        eprintln!("chmod: invalid option -- '{}'", c as char);
                        return Ok(1);
                    }
                }
            } else {
                pos.push(a);
            }
            i += 1;
        }
        if pos.len() < 2 {
            eprintln!("chmod: missing operand");
            return Ok(1);
        }
        let mode = match parse_octal(pos[0].as_bytes()) {
            Some(m) => m,
            None => {
                eprintln!("chmod: invalid mode '{}'", pos[0].to_string_lossy());
                return Ok(1);
            }
        };
        let mut rc = 0;
        for f in &pos[1..] {
            let p = Path::new(f);
            if recursive && fs::metadata(p).map(|m| m.is_dir()).unwrap_or(false) {
                if chmod_tree(p, mode) != 0 {
                    rc = 1;
                }
            } else if let Err(e) = do_chmod(p, mode) {
                eprintln!("chmod: {}: {}", p.display(), e);
                rc = 1;
            }
        }
        Ok(rc)
    }
}
