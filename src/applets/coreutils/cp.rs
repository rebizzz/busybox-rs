use crate::core::fs::{
    clean_parents_path, copy_file_entry, get_stat, last_path_component,
    CopyContext, CopyOptions,
};
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct CpApplet;
impl Applet for CpApplet {
    fn name(&self) -> &'static str {
        "cp"
    }
    fn description(&self) -> &'static str {
        "Copy SOURCE to DEST"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut set_a = false;
        let mut set_d = false;
        let mut set_p_flag = false;
        let mut set_l = false;
        let mut set_h = false;
        let mut recursive = false;
        let mut force = false;
        let mut interactive = false;
        let mut no_clobber = false;
        let mut make_hardlink = false;
        let mut make_symlink = false;
        let mut update = false;
        let mut verbose = false;
        let mut no_target_directory = false;
        let mut target_directory: Option<PathBuf> = None;
        let mut parents = false;
        let mut remove_destination = false;

        let mut positional = Vec::new();
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            let bytes = arg.as_bytes();
            if bytes == b"--" {
                for rem in iter {
                    positional.push(Path::new(rem));
                }
                break;
            } else if bytes.starts_with(b"--") {
                let opt = &bytes[2..];
                if opt == b"archive" {
                    set_a = true;
                    set_p_flag = true;
                    set_d = true;
                    recursive = true;
                } else if opt == b"force" {
                    force = true;
                } else if opt == b"interactive" {
                    interactive = true;
                    no_clobber = false;
                } else if opt == b"no-clobber" {
                    no_clobber = true;
                    interactive = false;
                } else if opt == b"link" {
                    make_hardlink = true;
                    make_symlink = false;
                } else if opt == b"symbolic-link" {
                    make_symlink = true;
                    make_hardlink = false;
                } else if opt == b"dereference" {
                    set_l = true;
                } else if opt == b"no-dereference" {
                    set_d = true;
                } else if opt == b"recursive" {
                    recursive = true;
                    set_d = true;
                } else if opt == b"verbose" {
                    verbose = true;
                } else if opt == b"update" {
                    update = true;
                } else if opt == b"parents" {
                    parents = true;
                } else if opt == b"no-target-directory" {
                    no_target_directory = true;
                } else if opt == b"remove-destination" {
                    remove_destination = true;
                    force = true;
                } else if opt.starts_with(b"target-directory=") {
                    let val = &opt[b"target-directory=".len()..];
                    target_directory = Some(PathBuf::from(std::ffi::OsStr::from_bytes(val)));
                } else if opt == b"target-directory" {
                    if let Some(next_arg) = iter.next() {
                        target_directory = Some(PathBuf::from(next_arg));
                    } else {
                        eprintln!("cp: option '--target-directory' requires an argument");
                        return Ok(1);
                    }
                } else if opt == b"preserve" {
                    set_p_flag = true;
                } else if opt.starts_with(b"preserve=") {
                    set_p_flag = true;
                    let val = &opt[b"preserve=".len()..];
                    if val.windows(5).any(|w| w == b"links") {
                        set_d = true;
                    }
                } else {
                    eprintln!(
                        "cp: unrecognized option '--{}'",
                        String::from_utf8_lossy(opt)
                    );
                    return Ok(1);
                }
            } else if bytes.starts_with(b"-") && bytes.len() > 1 {
                let mut idx = 1;
                while idx < bytes.len() {
                    let b = bytes[idx];
                    idx += 1;
                    match b {
                        b'a' => {
                            set_a = true;
                            set_p_flag = true;
                            set_d = true;
                            recursive = true;
                        }
                        b'd' => {
                            set_d = true;
                        }
                        b'P' => {
                            set_d = true;
                        }
                        b'L' => {
                            set_l = true;
                        }
                        b'H' => {
                            set_h = true;
                        }
                        b'p' => {
                            set_p_flag = true;
                        }
                        b'r' | b'R' => {
                            recursive = true;
                            set_d = true;
                        }
                        b'f' => {
                            force = true;
                        }
                        b'i' => {
                            interactive = true;
                            no_clobber = false;
                        }
                        b'n' => {
                            no_clobber = true;
                            interactive = false;
                        }
                        b'l' => {
                            make_hardlink = true;
                            make_symlink = false;
                        }
                        b's' => {
                            make_symlink = true;
                            make_hardlink = false;
                        }
                        b'u' => {
                            update = true;
                        }
                        b'v' => {
                            verbose = true;
                        }
                        b'T' => {
                            no_target_directory = true;
                        }
                        b't' => {
                            if idx < bytes.len() {
                                let val = &bytes[idx..];
                                target_directory =
                                    Some(PathBuf::from(std::ffi::OsStr::from_bytes(val)));
                                break;
                            } else if let Some(next_arg) = iter.next() {
                                target_directory = Some(PathBuf::from(next_arg));
                                break;
                            } else {
                                eprintln!("cp: option requires an argument -- 't'");
                                return Ok(1);
                            }
                        }
                        _ => {
                            eprintln!("cp: invalid option -- '{}'", b as char);
                            return Ok(1);
                        }
                    }
                }
            } else {
                positional.push(Path::new(arg));
            }
        }

        let dereference = if set_l { true } else { !set_d };
        let dereference_cmdline = set_h;
        let preserve_hardlinks = set_d || set_a || !dereference;

        let options = CopyOptions {
            preserve_status: set_p_flag,
            dereference,
            dereference_cmdline,
            recursive,
            force,
            interactive,
            no_clobber,
            make_hardlink,
            make_symlink,
            preserve_hardlinks,
            update,
            verbose,
            remove_destination,
        };

        let (sources, dest) = if let Some(ref target_dir) = target_directory {
            if positional.is_empty() {
                eprintln!("cp: missing file operand");
                return Ok(1);
            }
            (&positional[..], target_dir.as_path())
        } else {
            if positional.is_empty() {
                eprintln!("cp: missing file operand");
                return Ok(1);
            }
            if positional.len() < 2 {
                eprintln!(
                    "cp: missing destination file operand after '{}'",
                    positional[0].display()
                );
                return Ok(1);
            }
            (
                &positional[..positional.len() - 1],
                *positional.last().unwrap(),
            )
        };

        let dest_is_dir = match get_stat(dest, true) {
            Ok(st) => (st.st_mode & libc::S_IFMT) == libc::S_IFDIR,
            Err(_) => false,
        };

        if sources.len() > 1 && !dest_is_dir {
            eprintln!("cp: target '{}' is not a directory", dest.display());
            return Ok(1);
        }

        if parents && !dest_is_dir {
            eprintln!("cp: with --parents, the destination must be a directory");
            return Ok(1);
        }

        let mut context = CopyContext::new(options, "cp");
        let mut exit_code = 0;

        for src in sources {
            let target_path = if parents {
                dest.join(clean_parents_path(src))
            } else if dest_is_dir && !no_target_directory {
                dest.join(last_path_component(src))
            } else {
                dest.to_path_buf()
            };

            if parents {
                if let Some(parent) = target_path.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!("cp: can't create directory '{}': {}", parent.display(), e);
                        exit_code = 1;
                        continue;
                    }
                }
            }

            if copy_file_entry(src, &target_path, true, &mut context).is_err() {
                exit_code = 1;
            }
        }

        Ok(exit_code)
    }
}
