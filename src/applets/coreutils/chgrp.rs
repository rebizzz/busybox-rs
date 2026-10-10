use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct ChgrpApplet;
impl Applet for ChgrpApplet {
    fn name(&self) -> &'static str {
        "chgrp"
    }
    fn description(&self) -> &'static str {
        "Change file group"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut recursive = false;
        let mut noderef = false;
        let mut pos: Vec<&OsString> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
                for &c in &b[1..] {
                    match c {
                        b'R' => recursive = true,
                        b'h' => noderef = true,
                        _ => {
                            eprintln!("chgrp: invalid option -- '{}'", c as char);
                            return Ok(1);
                        }
                    }
                }
            } else {
                pos.push(a);
            }
        }
        if pos.len() < 2 {
            eprintln!("chgrp: missing operand");
            return Ok(1);
        }
        let gid = match resolve_gid(pos[0].as_bytes()) {
            Some(v) => v,
            None => {
                eprintln!("chgrp: invalid group '{}'", pos[0].to_string_lossy());
                return Ok(1);
            }
        };
        let mut rc = 0;
        for f in &pos[1..] {
            let p = Path::new(f);
            let r = if recursive {
                chown_tree(p, u32::MAX, gid, noderef)
            } else {
                match do_chown(p, u32::MAX, gid, noderef) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("chgrp: {}: {}", p.display(), e);
                        1
                    }
                }
            };
            if r != 0 {
                rc = 1;
            }
        }
        Ok(rc)
    }
}
