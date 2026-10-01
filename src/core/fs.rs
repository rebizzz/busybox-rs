use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use crate::core::errors::BbError;

/// Opens a path or standard input if path is "-" or empty
pub fn open_or_stdin(path: &Path) -> std::result::Result<Box<dyn Read>, BbError> {
    if path.as_os_str() == "-" {
        Ok(Box::new(io::stdin()))
    } else {
        match File::open(path) {
            Ok(f) => Ok(Box::new(f)),
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    Err(BbError::NotFound { path: path.to_path_buf() })
                } else if e.kind() == io::ErrorKind::PermissionDenied {
                    Err(BbError::PermissionDenied { path: path.to_path_buf() })
                } else {
                    Err(BbError::Io { path: Some(path.to_path_buf()), source: e })
                }
            }
        }
    }
}

pub fn read_bytes_or_stdin(path: &Path) -> std::result::Result<Vec<u8>, BbError> {
    let mut reader = open_or_stdin(path)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).map_err(|e| BbError::Io {
        path: Some(path.to_path_buf()),
        source: e,
    })?;
    Ok(buf)
}

/// Options controlling file and directory copying.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CopyOptions {
    pub preserve_status: bool,
    pub dereference: bool,
    pub dereference_cmdline: bool,
    pub recursive: bool,
    pub force: bool,
    pub interactive: bool,
    pub no_clobber: bool,
    pub make_hardlink: bool,
    pub make_symlink: bool,
    pub preserve_hardlinks: bool,
    pub update: bool,
    pub verbose: bool,
    pub parents: bool,
    pub remove_destination: bool,
    pub no_target_directory: bool,
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            preserve_status: false,
            dereference: true,
            dereference_cmdline: false,
            recursive: false,
            force: false,
            interactive: false,
            no_clobber: false,
            make_hardlink: false,
            make_symlink: false,
            preserve_hardlinks: false,
            update: false,
            verbose: false,
            parents: false,
            remove_destination: false,
            no_target_directory: false,
        }
    }
}

pub struct CopyContext {
    pub options: CopyOptions,
    pub applet_name: &'static str,
    pub hard_links: HashMap<(u64, u64), PathBuf>,
    pub created_dirs: HashSet<(u64, u64)>,
}

impl CopyContext {
    pub fn new(options: CopyOptions, applet_name: &'static str) -> Self {
        Self {
            options,
            applet_name,
            hard_links: HashMap::new(),
            created_dirs: HashSet::new(),
        }
    }
}

pub fn get_stat(path: &Path, follow_symlinks: bool) -> io::Result<libc::stat> {
    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let mut st = MaybeUninit::<libc::stat>::uninit();
    let res = unsafe {
        if follow_symlinks {
            libc::stat(c_path.as_ptr(), st.as_mut_ptr())
        } else {
            libc::lstat(c_path.as_ptr(), st.as_mut_ptr())
        }
    };
    if res < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(unsafe { st.assume_init() })
    }
}

pub fn last_path_component(path: &Path) -> &Path {
    let bytes = path.as_os_str().as_bytes();
    let mut end = bytes.len();
    while end > 1 && bytes[end - 1] == b'/' {
        end -= 1;
    }
    let trimmed = Path::new(std::ffi::OsStr::from_bytes(&bytes[..end]));
    trimmed.file_name().map(Path::new).unwrap_or(trimmed)
}

pub fn clean_parents_path(path: &Path) -> &Path {
    let bytes = path.as_os_str().as_bytes();
    let mut start = 0;
    while start < bytes.len() && bytes[start] == b'/' {
        start += 1;
    }
    Path::new(std::ffi::OsStr::from_bytes(&bytes[start..]))
}

pub fn copy_file_entry(
    src: &Path,
    dst: &Path,
    is_cmdline: bool,
    context: &mut CopyContext,
) -> Result<(), ()> {
    let follow_symlinks = if is_cmdline {
        context.options.dereference || context.options.dereference_cmdline
    } else {
        context.options.dereference
    };

    let src_stat = match get_stat(src, follow_symlinks) {
        Ok(st) => st,
        Err(e) => {
            eprintln!("{}: can't stat '{}': {}", context.applet_name, src.display(), e);
            return Err(());
        }
    };

    let is_dir = (src_stat.st_mode & libc::S_IFMT) == libc::S_IFDIR;

    if is_dir {
        if !context.options.recursive {
            eprintln!("{}: omitting directory '{}'", context.applet_name, src.display());
            return Err(());
        }

        let src_key = (src_stat.st_dev as u64, src_stat.st_ino as u64);
        if context.created_dirs.contains(&src_key) {
            eprintln!("{}: recursion detected, omitting directory '{}'", context.applet_name, src.display());
            return Err(());
        }

        let dst_stat_res = get_stat(dst, false);
        let dst_exists = dst_stat_res.is_ok();

        if let Ok(ref dst_stat) = dst_stat_res {
            if src_stat.st_dev == dst_stat.st_dev && src_stat.st_ino == dst_stat.st_ino {
                eprintln!("{}: '{}' and '{}' are the same file", context.applet_name, src.display(), dst.display());
                return Err(());
            }
            if (dst_stat.st_mode & libc::S_IFMT) != libc::S_IFDIR {
                eprintln!("{}: target '{}' is not a directory", context.applet_name, dst.display());
                return Err(());
            }
        } else {
            let saved_umask = unsafe { libc::umask(0) };
            let mut mode = (src_stat.st_mode & 0o7777) as libc::mode_t;
            if !context.options.preserve_status {
                mode &= !saved_umask;
            }
            mode |= libc::S_IRWXU;
            let c_dst = match CString::new(dst.as_os_str().as_bytes()) {
                Ok(c) => c,
                Err(_) => {
                    unsafe { libc::umask(saved_umask) };
                    return Err(());
                }
            };
            let res = unsafe { libc::mkdir(c_dst.as_ptr(), mode) };
            unsafe { libc::umask(saved_umask) };
            if res < 0 {
                eprintln!("{}: can't create directory '{}': {}", context.applet_name, dst.display(), io::Error::last_os_error());
                return Err(());
            }
            if let Ok(new_dst_stat) = get_stat(dst, false) {
                context.created_dirs.insert((new_dst_stat.st_dev as u64, new_dst_stat.st_ino as u64));
            }
        }

        let entries = match fs::read_dir(src) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("{}: can't read directory '{}': {}", context.applet_name, src.display(), e);
                return Err(());
            }
        };

        let mut child_err = false;
        let mut entry_names = Vec::new();
        for entry in entries {
            match entry {
                Ok(e) => entry_names.push(e.file_name()),
                Err(_) => child_err = true,
            }
        }
        entry_names.sort();

        for name in entry_names {
            let child_src = src.join(&name);
            let child_dst = dst.join(&name);
            if copy_file_entry(&child_src, &child_dst, false, context).is_err() {
                child_err = true;
            }
        }

        let c_dst = match CString::new(dst.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => return Err(()),
        };

        if !dst_exists {
            if !context.options.preserve_status {
                let saved_umask = unsafe { libc::umask(0) };
                unsafe { libc::umask(saved_umask) };
                let _ = unsafe { libc::chmod(c_dst.as_ptr(), (src_stat.st_mode & 0o7777) & !saved_umask) };
            } else {
                let _ = unsafe { libc::chown(c_dst.as_ptr(), src_stat.st_uid, src_stat.st_gid) };
                let _ = unsafe { libc::chmod(c_dst.as_ptr(), src_stat.st_mode & 0o7777) };
                let times = [
                    libc::timespec { tv_sec: src_stat.st_atime, tv_nsec: src_stat.st_atime_nsec },
                    libc::timespec { tv_sec: src_stat.st_mtime, tv_nsec: src_stat.st_mtime_nsec },
                ];
                let _ = unsafe { libc::utimensat(libc::AT_FDCWD, c_dst.as_ptr(), times.as_ptr(), 0) };
            }
        } else if context.options.preserve_status {
            let _ = unsafe { libc::chown(c_dst.as_ptr(), src_stat.st_uid, src_stat.st_gid) };
            let _ = unsafe { libc::chmod(c_dst.as_ptr(), src_stat.st_mode & 0o7777) };
            let times = [
                libc::timespec { tv_sec: src_stat.st_atime, tv_nsec: src_stat.st_atime_nsec },
                libc::timespec { tv_sec: src_stat.st_mtime, tv_nsec: src_stat.st_mtime_nsec },
            ];
            let _ = unsafe { libc::utimensat(libc::AT_FDCWD, c_dst.as_ptr(), times.as_ptr(), 0) };
        }

        if context.options.verbose {
            println!("'{}' -> '{}'", src.display(), dst.display());
        }

        if child_err {
            return Err(());
        }
        return Ok(());
    }

    let dst_stat_res = get_stat(dst, false);
    let dst_exists = dst_stat_res.is_ok();

    if let Ok(ref dst_stat) = dst_stat_res {
        if src_stat.st_dev == dst_stat.st_dev && src_stat.st_ino == dst_stat.st_ino {
            eprintln!("{}: '{}' and '{}' are the same file", context.applet_name, src.display(), dst.display());
            return Err(());
        }
        if context.options.no_clobber {
            return Ok(());
        }
        if context.options.update && src_stat.st_mtime <= dst_stat.st_mtime {
            return Ok(());
        }
        if context.options.interactive {
            eprint!("{}: overwrite '{}'? ", context.applet_name, dst.display());
            let mut resp = String::new();
            if io::stdin().read_line(&mut resp).is_err() || !(resp.starts_with('y') || resp.starts_with('Y')) {
                return Ok(());
            }
        }
        if context.options.remove_destination {
            let _ = fs::remove_file(dst);
        }
    }

    let is_lnk = (src_stat.st_mode & libc::S_IFMT) == libc::S_IFLNK;

    if context.options.preserve_hardlinks && !follow_symlinks {
        let key = (src_stat.st_dev as u64, src_stat.st_ino as u64);
        if let Some(target) = context.hard_links.get(&key) {
            if dst_exists && (context.options.force || context.options.remove_destination) {
                let _ = fs::remove_file(dst);
            }
            match fs::hard_link(target, dst) {
                Ok(_) => {
                    if context.options.verbose {
                        println!("'{}' -> '{}'", src.display(), dst.display());
                    }
                    return Ok(());
                }
                Err(_) => {
                    let _ = fs::remove_file(dst);
                    if fs::hard_link(target, dst).is_ok() {
                        if context.options.verbose {
                            println!("'{}' -> '{}'", src.display(), dst.display());
                        }
                        return Ok(());
                    }
                }
            }
        }
    }

    if context.options.make_symlink {
        if dst_exists && (context.options.force || context.options.remove_destination) {
            let _ = fs::remove_file(dst);
        }
        if let Err(e) = std::os::unix::fs::symlink(src, dst) {
            if context.options.force {
                let _ = fs::remove_file(dst);
                if let Err(e2) = std::os::unix::fs::symlink(src, dst) {
                    eprintln!("{}: can't create symlink '{}': {}", context.applet_name, dst.display(), e2);
                    return Err(());
                }
            } else {
                eprintln!("{}: can't create symlink '{}': {}", context.applet_name, dst.display(), e);
                return Err(());
            }
        }
        if context.options.verbose {
            println!("'{}' -> '{}'", src.display(), dst.display());
        }
        return Ok(());
    }

    if context.options.make_hardlink {
        if dst_exists && (context.options.force || context.options.remove_destination) {
            let _ = fs::remove_file(dst);
        }
        if let Err(e) = fs::hard_link(src, dst) {
            if context.options.force {
                let _ = fs::remove_file(dst);
                if let Err(e2) = fs::hard_link(src, dst) {
                    eprintln!("{}: can't create link '{}': {}", context.applet_name, dst.display(), e2);
                    return Err(());
                }
            } else {
                eprintln!("{}: can't create link '{}': {}", context.applet_name, dst.display(), e);
                return Err(());
            }
        }
        if context.options.verbose {
            println!("'{}' -> '{}'", src.display(), dst.display());
        }
        return Ok(());
    }

    if is_lnk {
        let link_target = match fs::read_link(src) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}: can't readlink '{}': {}", context.applet_name, src.display(), e);
                return Err(());
            }
        };
        if dst_exists {
            let _ = fs::remove_file(dst);
        }
        if let Err(e) = std::os::unix::fs::symlink(&link_target, dst) {
            eprintln!("{}: can't create symlink '{}': {}", context.applet_name, dst.display(), e);
            return Err(());
        }
        let c_dst = match CString::new(dst.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => return Err(()),
        };
        if context.options.preserve_status {
            unsafe { libc::lchown(c_dst.as_ptr(), src_stat.st_uid, src_stat.st_gid); }
        }
        if context.options.preserve_hardlinks && !follow_symlinks {
            context.hard_links.insert((src_stat.st_dev as u64, src_stat.st_ino as u64), dst.to_path_buf());
        }
        if context.options.verbose {
            println!("'{}' -> '{}'", src.display(), dst.display());
        }
        return Ok(());
    }

    let is_chr = (src_stat.st_mode & libc::S_IFMT) == libc::S_IFCHR;
    let is_blk = (src_stat.st_mode & libc::S_IFMT) == libc::S_IFBLK;
    let is_fifo = (src_stat.st_mode & libc::S_IFMT) == libc::S_IFIFO;
    let is_sock = (src_stat.st_mode & libc::S_IFMT) == libc::S_IFSOCK;

    if context.options.recursive && (is_chr || is_blk || is_fifo || is_sock) {
        if dst_exists {
            let _ = fs::remove_file(dst);
        }
        let c_dst = match CString::new(dst.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => return Err(()),
        };
        let ret = unsafe { libc::mknod(c_dst.as_ptr(), src_stat.st_mode, src_stat.st_rdev) };
        if ret < 0 {
            eprintln!("{}: can't create special file '{}': {}", context.applet_name, dst.display(), io::Error::last_os_error());
            return Err(());
        }
        if context.options.preserve_status {
            unsafe {
                let _ = libc::chown(c_dst.as_ptr(), src_stat.st_uid, src_stat.st_gid);
                let _ = libc::chmod(c_dst.as_ptr(), src_stat.st_mode & 0o7777);
                let times = [
                    libc::timespec { tv_sec: src_stat.st_atime, tv_nsec: src_stat.st_atime_nsec },
                    libc::timespec { tv_sec: src_stat.st_mtime, tv_nsec: src_stat.st_mtime_nsec },
                ];
                let _ = libc::utimensat(libc::AT_FDCWD, c_dst.as_ptr(), times.as_ptr(), 0);
            }
        }
        if context.options.preserve_hardlinks && !follow_symlinks {
            context.hard_links.insert((src_stat.st_dev as u64, src_stat.st_ino as u64), dst.to_path_buf());
        }
        if context.options.verbose {
            println!("'{}' -> '{}'", src.display(), dst.display());
        }
        return Ok(());
    }

    let mut src_file = match File::open(src) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}: can't open '{}': {}", context.applet_name, src.display(), e);
            return Err(());
        }
    };

    let is_reg = (src_stat.st_mode & libc::S_IFMT) == libc::S_IFREG;
    let new_mode = if is_reg {
        (src_stat.st_mode & 0o7777) as u32
    } else {
        0o666
    };

    let mut open_opts = OpenOptions::new();
    open_opts.write(true).create(true).truncate(true).mode(new_mode);

    let mut dst_file = match open_opts.open(dst) {
        Ok(f) => f,
        Err(e) => {
            if context.options.force || context.options.remove_destination {
                let _ = fs::remove_file(dst);
                match open_opts.open(dst) {
                    Ok(f) => f,
                    Err(e2) => {
                        eprintln!("{}: can't create '{}': {}", context.applet_name, dst.display(), e2);
                        return Err(());
                    }
                }
            } else {
                eprintln!("{}: can't create '{}': {}", context.applet_name, dst.display(), e);
                return Err(());
            }
        }
    };

    let mut buf = [0u8; 64 * 1024];
    let mut write_failed = false;
    loop {
        match src_file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if let Err(e) = dst_file.write_all(&buf[..n]) {
                    eprintln!("{}: error writing to '{}': {}", context.applet_name, dst.display(), e);
                    write_failed = true;
                    break;
                }
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => {
                eprintln!("{}: error reading from '{}': {}", context.applet_name, src.display(), e);
                write_failed = true;
                break;
            }
        }
    }
    drop(dst_file);
    drop(src_file);

    if write_failed {
        return Err(());
    }

    let c_dst = match CString::new(dst.as_os_str().as_bytes()) {
        Ok(c) => c,
        Err(_) => return Err(()),
    };

    if is_reg {
        if context.options.preserve_status {
            unsafe {
                let _ = libc::chown(c_dst.as_ptr(), src_stat.st_uid, src_stat.st_gid);
                let _ = libc::chmod(c_dst.as_ptr(), src_stat.st_mode & 0o7777);
                let times = [
                    libc::timespec { tv_sec: src_stat.st_atime, tv_nsec: src_stat.st_atime_nsec },
                    libc::timespec { tv_sec: src_stat.st_mtime, tv_nsec: src_stat.st_mtime_nsec },
                ];
                let _ = libc::utimensat(libc::AT_FDCWD, c_dst.as_ptr(), times.as_ptr(), 0);
            }
        }
    }

    if context.options.preserve_hardlinks && !follow_symlinks {
        context.hard_links.insert((src_stat.st_dev as u64, src_stat.st_ino as u64), dst.to_path_buf());
    }

    if context.options.verbose {
        println!("'{}' -> '{}'", src.display(), dst.display());
    }

    Ok(())
}

/// Recursively copy a file or directory preserving Unix semantics
pub fn copy_recursive(src: &Path, dst: &Path) -> std::result::Result<(), BbError> {
    let options = CopyOptions {
        recursive: true,
        dereference: false,
        preserve_status: true,
        preserve_hardlinks: true,
        force: true,
        ..Default::default()
    };
    let mut context = CopyContext::new(options, "cp");
    copy_file_entry(src, dst, false, &mut context).map_err(|_| {
        BbError::Io {
            path: Some(src.to_path_buf()),
            source: io::Error::new(io::ErrorKind::Other, "copy failed"),
        }
    })
}
