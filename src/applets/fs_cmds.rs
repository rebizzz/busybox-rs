use crate::core::fs::{
    clean_parents_path, copy_file_entry, copy_recursive, get_stat, last_path_component,
    CopyContext, CopyOptions,
};
use crate::core::{Applet, Result};
use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct LsApplet;
impl Applet for LsApplet {
    fn name(&self) -> &'static str {
        "ls"
    }
    fn description(&self) -> &'static str {
        "List directory contents"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut show_all = false;
        let mut paths = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                for &b in &bytes[1..] {
                    if b == b'a' {
                        show_all = true;
                    }
                }
            } else {
                paths.push(Path::new(arg));
            }
        }

        if paths.is_empty() {
            paths.push(Path::new("."));
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let multiple = paths.len() > 1;
        let mut exit_code = 0;

        for (i, p) in paths.iter().enumerate() {
            if multiple {
                if i > 0 {
                    handle.write_all(b"\n")?;
                }
                handle.write_all(p.as_os_str().as_bytes())?;
                handle.write_all(b":\n")?;
            }

            match fs::metadata(p) {
                Ok(meta) if meta.is_dir() => {
                    let mut entries = Vec::new();
                    match fs::read_dir(p) {
                        Ok(dir) => {
                            for e in dir.flatten() {
                                let name = e.file_name();
                                if !show_all && name.as_bytes().starts_with(b".") {
                                    continue;
                                }
                                entries.push(name);
                            }
                            entries.sort();
                            for entry in entries {
                                handle.write_all(entry.as_bytes())?;
                                handle.write_all(b"\n")?;
                            }
                        }
                        Err(e) => {
                            eprintln!("ls: {}: {}", p.display(), e);
                            exit_code = 1;
                        }
                    }
                }
                Ok(_) => {
                    handle.write_all(p.as_os_str().as_bytes())?;
                    handle.write_all(b"\n")?;
                }
                Err(e) => {
                    eprintln!("ls: {}: {}", p.display(), e);
                    exit_code = 1;
                }
            }
        }
        Ok(exit_code)
    }
}

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

pub struct MvApplet;
impl Applet for MvApplet {
    fn name(&self) -> &'static str {
        "mv"
    }
    fn description(&self) -> &'static str {
        "Rename SOURCE to DEST, or move SOURCE(s) to DIRECTORY"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut targets = Vec::new();
        for arg in args {
            let bytes = arg.as_bytes();
            if !bytes.starts_with(b"-") || bytes == b"-" {
                targets.push(Path::new(arg));
            }
        }

        if targets.len() < 2 {
            eprintln!("mv: missing file operand");
            return Ok(1);
        }

        let dst = targets.last().unwrap();
        let sources = &targets[..targets.len() - 1];
        let dst_is_dir = fs::metadata(dst).map(|m| m.is_dir()).unwrap_or(false);

        if sources.len() > 1 && !dst_is_dir {
            eprintln!("mv: target '{}' is not a directory", dst.display());
            return Ok(1);
        }

        for src in sources {
            let target_path = if dst_is_dir {
                let name = src.file_name().unwrap_or(src.as_os_str());
                dst.join(name)
            } else {
                dst.to_path_buf()
            };

            if fs::rename(src, &target_path).is_err() {
                // If cross-device, copy and remove
                copy_recursive(src, &target_path)?;
                if fs::metadata(src).map(|m| m.is_dir()).unwrap_or(false) {
                    fs::remove_dir_all(src)?;
                } else {
                    fs::remove_file(src)?;
                }
            }
        }
        Ok(0)
    }
}

pub struct RmApplet;
impl Applet for RmApplet {
    fn name(&self) -> &'static str {
        "rm"
    }
    fn description(&self) -> &'static str {
        "Remove (unlink) the FILE(s)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut recursive = false;
        let mut force = false;
        let mut paths = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                for &b in &bytes[1..] {
                    match b {
                        b'r' | b'R' => recursive = true,
                        b'f' => force = true,
                        _ => {}
                    }
                }
            } else {
                paths.push(Path::new(arg));
            }
        }

        let mut exit_code = 0;
        for p in paths {
            let res = if recursive {
                if p.is_dir() {
                    fs::remove_dir_all(p)
                } else {
                    fs::remove_file(p)
                }
            } else {
                fs::remove_file(p)
            };

            if let Err(e) = res {
                if !force {
                    eprintln!("rm: cannot remove '{}': {}", p.display(), e);
                    exit_code = 1;
                }
            }
        }
        Ok(exit_code)
    }
}

pub struct MkdirApplet;
impl Applet for MkdirApplet {
    fn name(&self) -> &'static str {
        "mkdir"
    }
    fn description(&self) -> &'static str {
        "Create the DIRECTORY(ies)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut parents = false;
        let mut dirs = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 {
                for &b in &bytes[1..] {
                    if b == b'p' {
                        parents = true;
                    }
                }
            } else {
                dirs.push(Path::new(arg));
            }
        }

        if dirs.is_empty() {
            eprintln!("mkdir: missing operand");
            return Ok(1);
        }

        let mut exit_code = 0;
        for d in dirs {
            let res = if parents {
                fs::create_dir_all(d)
            } else {
                fs::create_dir(d)
            };
            if let Err(e) = res {
                eprintln!("mkdir: cannot create directory '{}': {}", d.display(), e);
                exit_code = 1;
            }
        }
        Ok(exit_code)
    }
}

pub struct RmdirApplet;
impl Applet for RmdirApplet {
    fn name(&self) -> &'static str {
        "rmdir"
    }
    fn description(&self) -> &'static str {
        "Remove EMPTY DIRECTORY(ies)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dirs = Vec::new();
        for arg in args {
            if !arg.as_bytes().starts_with(b"-") {
                dirs.push(Path::new(arg));
            }
        }

        let mut exit_code = 0;
        for d in dirs {
            if let Err(e) = fs::remove_dir(d) {
                eprintln!("rmdir: failed to remove '{}': {}", d.display(), e);
                exit_code = 1;
            }
        }
        Ok(exit_code)
    }
}

pub struct TouchApplet;
impl Applet for TouchApplet {
    fn name(&self) -> &'static str {
        "touch"
    }
    fn description(&self) -> &'static str {
        "Update the access and modification times of each FILE to the current time"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut files = Vec::new();
        for arg in args {
            if !arg.as_bytes().starts_with(b"-") {
                files.push(Path::new(arg));
            }
        }

        let mut exit_code = 0;
        for f in files {
            match OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(false)
                .open(f)
            {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("touch: cannot touch '{}': {}", f.display(), e);
                    exit_code = 1;
                }
            }
        }
        Ok(exit_code)
    }
}

pub struct LinkApplet;
impl Applet for LinkApplet {
    fn name(&self) -> &'static str {
        "link"
    }
    fn description(&self) -> &'static str {
        "Create a link to FILE with the name LINK_NAME"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() != 2 {
            eprintln!("link: expected 2 arguments");
            return Ok(1);
        }
        fs::hard_link(Path::new(&args[0]), Path::new(&args[1]))?;
        Ok(0)
    }
}

pub struct UnlinkApplet;
impl Applet for UnlinkApplet {
    fn name(&self) -> &'static str {
        "unlink"
    }
    fn description(&self) -> &'static str {
        "Call the unlink function to remove the specified FILE"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() != 1 {
            eprintln!("unlink: expected 1 argument");
            return Ok(1);
        }
        fs::remove_file(Path::new(&args[0]))?;
        Ok(0)
    }
}

pub struct DirnameApplet;
impl Applet for DirnameApplet {
    fn name(&self) -> &'static str {
        "dirname"
    }
    fn description(&self) -> &'static str {
        "Strip last component from file name"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("dirname: missing operand");
            return Ok(1);
        }
        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.is_empty() {
                println!(".");
                continue;
            }
            let mut end = bytes.len();
            while end > 0 && bytes[end - 1] == b'/' {
                end -= 1;
            }
            if end == 0 {
                println!("/");
            } else {
                let trimmed = &bytes[..end];
                if let Some(pos) = trimmed.iter().rposition(|&b| b == b'/') {
                    let mut p_end = pos;
                    while p_end > 0 && trimmed[p_end - 1] == b'/' {
                        p_end -= 1;
                    }
                    if p_end == 0 {
                        println!("/");
                    } else {
                        let stdout = io::stdout();
                        let mut handle = stdout.lock();
                        handle.write_all(&trimmed[..p_end])?;
                        handle.write_all(b"\n")?;
                    }
                } else {
                    println!(".");
                }
            }
        }
        Ok(0)
    }
}

pub struct BasenameApplet;
impl Applet for BasenameApplet {
    fn name(&self) -> &'static str {
        "basename"
    }
    fn description(&self) -> &'static str {
        "Strip directory and suffix from filenames"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("basename: missing operand");
            return Ok(1);
        }
        let bytes = args[0].as_bytes();
        let mut end = bytes.len();
        while end > 0 && bytes[end - 1] == b'/' {
            end -= 1;
        }
        if end == 0 {
            println!("/");
            return Ok(0);
        }
        let trimmed = &bytes[..end];
        let name_bytes = if let Some(pos) = trimmed.iter().rposition(|&b| b == b'/') {
            &trimmed[pos + 1..]
        } else {
            trimmed
        };

        let mut out = name_bytes;
        if args.len() > 1 && !args[1].is_empty() {
            let suffix = args[1].as_bytes();
            if out.ends_with(suffix) && out.len() > suffix.len() {
                out = &out[..out.len() - suffix.len()];
            }
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(out)?;
        handle.write_all(b"\n")?;
        Ok(0)
    }
}

pub struct WhichApplet;
impl Applet for WhichApplet {
    fn name(&self) -> &'static str {
        "which"
    }
    fn description(&self) -> &'static str {
        "Locate a command"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            return Ok(1);
        }
        let path_var = env::var_os("PATH").unwrap_or_default();
        let paths: Vec<&[u8]> = path_var.as_bytes().split(|&b| b == b':').collect();
        let mut ret = 0;

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for cmd in args {
            let cmd_bytes = cmd.as_bytes();
            let mut found = false;
            if cmd_bytes.contains(&b'/') {
                let p = Path::new(cmd);
                if p.is_file() {
                    handle.write_all(p.as_os_str().as_bytes())?;
                    handle.write_all(b"\n")?;
                    found = true;
                }
            } else {
                for dir in &paths {
                    let mut buf = dir.to_vec();
                    buf.push(b'/');
                    buf.extend_from_slice(cmd_bytes);
                    use std::os::unix::ffi::OsStrExt;
                    let p = Path::new(std::ffi::OsStr::from_bytes(&buf));
                    if p.is_file() {
                        handle.write_all(p.as_os_str().as_bytes())?;
                        handle.write_all(b"\n")?;
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                ret = 1;
            }
        }
        Ok(ret)
    }
}
