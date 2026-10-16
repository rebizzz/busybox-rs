use crate::core::{Applet, Result};
use std::ffi::{OsStr, OsString};
use std::fs::{self};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct LnApplet;
impl Applet for LnApplet {
    fn name(&self) -> &'static str {
        "ln"
    }
    fn description(&self) -> &'static str {
        "Make links between files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sym = false;
        let mut force = false;
        let mut verbose = false;
        let mut no_target_dir = false;
        let mut backup = false;
        let mut pos: Vec<&Path> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for r in &args[i + 1..] {
                    pos.push(Path::new(r));
                }
                break;
            } else if b.starts_with(b"-") && b.len() > 1 && b != b"-" {
                let mut j = 1;
                while j < b.len() {
                    match b[j] {
                        b's' => sym = true,
                        b'f' => force = true,
                        b'v' => verbose = true,
                        b'T' => no_target_dir = true,
                        b'n' => no_target_dir = true,
                        b'b' => backup = true,
                        b'S' => {
                            backup = true;
                            if j + 1 >= b.len() {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("ln: option requires an argument -- 'S'");
                                    return Ok(1);
                                }
                            }
                            break;
                        }
                        _ => {
                            eprintln!("ln: invalid option -- '{}'", b[j] as char);
                            return Ok(1);
                        }
                    }
                    j += 1;
                }
            } else {
                pos.push(Path::new(&args[i]));
            }
            i += 1;
        }
        if pos.len() < 2 {
            eprintln!("ln: missing file operand");
            return Ok(1);
        }
        let (srcs, dst) = (pos[..pos.len() - 1].to_vec(), pos[pos.len() - 1]);
        let dst_is_dir = if no_target_dir {
            false
        } else {
            fs::symlink_metadata(dst)
                .map(|m| m.is_dir())
                .unwrap_or(false)
        };
        if srcs.len() > 1 && !dst_is_dir {
            eprintln!("ln: target '{}' is not a directory", dst.display());
            return Ok(1);
        }
        let stdout = io::stdout();
        let mut out = stdout.lock();
        ln_all(
            &mut out, &srcs, dst, dst_is_dir, sym, backup, force, verbose,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn ln_all(
    out: &mut impl Write,
    srcs: &[&Path],
    dst: &Path,
    dst_is_dir: bool,
    sym: bool,
    backup: bool,
    force: bool,
    verbose: bool,
) -> Result<i32> {
    let mut rc = 0;
    for s in srcs {
        let target = if dst_is_dir {
            dst.join(s.file_name().unwrap_or(s.as_os_str()))
        } else {
            dst.to_path_buf()
        };
        if backup && target.exists() {
            let mut bkp = target.as_os_str().as_bytes().to_vec();
            bkp.push(b'~');
            let _ = fs::rename(&target, Path::new(OsStr::from_bytes(&bkp)));
        } else if force {
            let _ = fs::remove_file(&target);
        }
        let r = if sym {
            std::os::unix::fs::symlink(s, &target)
        } else {
            fs::hard_link(s, &target)
        };
        if let Err(e) = r {
            eprintln!("ln: failed to create link '{}': {}", target.display(), e);
            rc = 1;
        } else if verbose {
            out.write_all(target.as_os_str().as_bytes())?;
            out.write_all(b" -> ")?;
            out.write_all(s.as_os_str().as_bytes())?;
            out.write_all(b"\n")?;
        }
    }
    Ok(rc)
}
