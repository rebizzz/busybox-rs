use crate::core::{Applet, Result};
use std::collections::HashMap;
use std::ffi::{CStr, CString, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};

fn print_bytes(bytes: &[u8]) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(bytes);
}

fn put_num(buf: &mut Vec<u8>, mut n: u64) {
    if n == 0 {
        buf.push(b'0');
        return;
    }
    let mut t = [0u8; 20];
    let mut i = t.len();
    while n > 0 {
        i -= 1;
        t[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    buf.extend_from_slice(&t[i..]);
}

fn put_num_pad_left(buf: &mut Vec<u8>, n: u64, width: usize) {
    let start = buf.len();
    put_num(buf, n);
    let len = buf.len() - start;
    if len < width {
        let pad = width - len;
        let digits = buf[start..].to_vec();
        buf.truncate(start);
        for _ in 0..pad {
            buf.push(b' ');
        }
        buf.extend_from_slice(&digits);
    }
}

fn split_byte_slice(s: &[u8], delim: u8) -> Option<(&[u8], &[u8])> {
    s.iter()
        .position(|&b| b == delim)
        .map(|pos| (&s[..pos], &s[pos + 1..]))
}

fn get_kernel_release() -> String {
    let mut uts: libc::utsname = unsafe { std::mem::zeroed() };
    if unsafe { libc::uname(&mut uts) } == 0 {
        let r = unsafe { CStr::from_ptr(uts.release.as_ptr()) };
        return r.to_string_lossy().into_owned();
    }
    "custom".to_string()
}

pub struct LsmodApplet;

impl Applet for LsmodApplet {
    fn name(&self) -> &'static str {
        "lsmod"
    }
    fn description(&self) -> &'static str {
        "List loaded kernel modules"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let content = match fs::read("/proc/modules") {
            Ok(c) => c,
            Err(e) => {
                eprintln!("lsmod: cannot open /proc/modules: {}", e);
                return Ok(1);
            }
        };

        let mut out = Vec::new();
        out.extend_from_slice(b"Module                  Size  Used by\n");

        for line in content.split(|&b| b == b'\n') {
            if line.is_empty() {
                continue;
            }
            let fields: Vec<&[u8]> = line
                .split(|&b| b == b' ' || b == b'\t')
                .filter(|s| !s.is_empty())
                .collect();
            if fields.len() < 3 {
                continue;
            }
            let name = fields[0];
            let size = fields[1];
            let refcnt = fields[2];
            let used_by = if fields.len() > 3 && fields[3] != b"-" {
                let s = fields[3];
                if s.ends_with(b",") {
                    &s[..s.len() - 1]
                } else {
                    s
                }
            } else {
                b""
            };

            out.extend_from_slice(name);
            let n_len = name.len();
            if n_len < 19 {
                out.resize(out.len() + 19 - n_len, b' ');
            }
            out.push(b' ');

            let s_len = size.len();
            if s_len < 8 {
                out.resize(out.len() + 8 - s_len, b' ');
            }
            out.extend_from_slice(size);
            out.push(b' ');

            let r_len = refcnt.len();
            if r_len < 2 {
                out.resize(out.len() + 2 - r_len, b' ');
            }
            out.extend_from_slice(refcnt);

            if !used_by.is_empty() {
                out.push(b' ');
                out.extend_from_slice(used_by);
            }
            out.push(b'\n');
        }

        print_bytes(&out);
        Ok(0)
    }
}

pub struct InsmodApplet;

impl Applet for InsmodApplet {
    fn name(&self) -> &'static str {
        "insmod"
    }
    fn description(&self) -> &'static str {
        "Load kernel module into the kernel"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("insmod: filename required");
            return Ok(1);
        }

        let path = Path::new(&args[0]);
        let mut opts = Vec::new();
        for (i, a) in args[1..].iter().enumerate() {
            if i > 0 {
                opts.push(b' ');
            }
            opts.extend_from_slice(a.as_bytes());
        }
        let opts_cstr = CString::new(opts).unwrap_or_else(|_| CString::new("").unwrap());

        let f = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("insmod: can't open '{}': {}", path.display(), e);
                return Ok(1);
            }
        };

        let fd = f.as_raw_fd();
        let ret = unsafe { libc::syscall(libc::SYS_finit_module, fd, opts_cstr.as_ptr(), 0) };

        if ret == 0 {
            return Ok(0);
        }

        let data = match fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("insmod: can't read '{}': {}", path.display(), e);
                return Ok(1);
            }
        };

        let ret = unsafe {
            libc::syscall(
                libc::SYS_init_module,
                data.as_ptr() as *const libc::c_void,
                data.len() as libc::size_t,
                opts_cstr.as_ptr(),
            )
        };

        if ret != 0 {
            let err = io::Error::last_os_error();
            eprintln!("insmod: can't insert '{}': {}", path.display(), err);
            return Ok(1);
        }

        Ok(0)
    }
}

pub struct RmmodApplet;

impl Applet for RmmodApplet {
    fn name(&self) -> &'static str {
        "rmmod"
    }
    fn description(&self) -> &'static str {
        "Unload kernel module"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut force = false;
        let mut mod_name: Option<&[u8]> = None;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-f" || b == b"--force" {
                force = true;
            } else if !b.is_empty() && b[0] == b'-' {
            } else if mod_name.is_none() {
                mod_name = Some(b);
            }
        }

        let name = match mod_name {
            Some(n) => n,
            None => {
                eprintln!("rmmod: module name required");
                return Ok(1);
            }
        };

        let name_cstr = match CString::new(name) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("rmmod: invalid module name");
                return Ok(1);
            }
        };

        let flags = if force {
            libc::O_TRUNC | libc::O_NONBLOCK
        } else {
            libc::O_NONBLOCK
        };
        let ret = unsafe { libc::syscall(libc::SYS_delete_module, name_cstr.as_ptr(), flags) };

        if ret != 0 {
            let err = io::Error::last_os_error();
            eprintln!(
                "rmmod: can't unload '{}': {}",
                String::from_utf8_lossy(name),
                err
            );
            return Ok(1);
        }

        Ok(0)
    }
}

pub struct ModprobeApplet;

impl Applet for ModprobeApplet {
    fn name(&self) -> &'static str {
        "modprobe"
    }
    fn description(&self) -> &'static str {
        "Add and remove modules from the Linux Kernel"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut remove = false;
        let mut target: Option<String> = None;
        let mut opts = Vec::new();

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-r" || b == b"--remove" {
                remove = true;
            } else if !b.is_empty() && b[0] == b'-' {
            } else if target.is_none() {
                target = Some(String::from_utf8_lossy(b).into_owned());
            } else {
                opts.push(arg.clone());
            }
        }

        let module = match target {
            Some(m) => m,
            None => {
                eprintln!("modprobe: module name required");
                return Ok(1);
            }
        };

        let rel = get_kernel_release();
        let dep_file = PathBuf::from(format!("/lib/modules/{}/modules.dep", rel));

        if remove {
            let app = RmmodApplet;
            return app.run(&[OsString::from(module)]);
        }

        let deps = parse_modules_dep(&dep_file);

        let norm_module = module.replace('-', "_");

        let mut to_load = Vec::new();
        let mut found_path: Option<String> = None;

        for (k, v) in &deps {
            let base = Path::new(k)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .replace('-', "_");
            if base == norm_module {
                found_path = Some(k.clone());
                for dep in v {
                    to_load.push(dep.clone());
                }
                break;
            }
        }

        if let Some(p) = found_path {
            to_load.reverse();
            let base_dir = PathBuf::from(format!("/lib/modules/{}", rel));
            for dep in to_load {
                let full_path = if dep.starts_with('/') {
                    PathBuf::from(dep)
                } else {
                    base_dir.join(dep)
                };
                let ins = InsmodApplet;
                let _ = ins.run(&[full_path.into_os_string()]);
            }

            let full_path = if p.starts_with('/') {
                PathBuf::from(p)
            } else {
                base_dir.join(p)
            };
            let mut ins_args = vec![full_path.into_os_string()];
            ins_args.extend(opts);
            let ins = InsmodApplet;
            ins.run(&ins_args)
        } else {
            let p = Path::new(&module);
            if p.exists() {
                let mut ins_args = vec![p.as_os_str().to_os_string()];
                ins_args.extend(opts);
                let ins = InsmodApplet;
                return ins.run(&ins_args);
            }
            eprintln!(
                "modprobe: module '{}' not found in /lib/modules/{}",
                module, rel
            );
            Ok(1)
        }
    }
}

fn parse_modules_dep(path: &Path) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    let f = match File::open(path) {
        Ok(f) => f,
        Err(_) => return map,
    };
    let reader = io::BufReader::new(f);
    for line in reader.lines().map_while(|l| l.ok()) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((mod_part, dep_part)) = trimmed.split_once(':') {
            let mod_path = mod_part.trim().to_string();
            let deps: Vec<String> = dep_part.split_whitespace().map(|s| s.to_string()).collect();
            map.insert(mod_path, deps);
        }
    }
    map
}

pub struct DepmodApplet;

impl Applet for DepmodApplet {
    fn name(&self) -> &'static str {
        "depmod"
    }
    fn description(&self) -> &'static str {
        "Generate modules.dep and map files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut rel: Option<String> = None;
        for arg in args {
            let b = arg.as_bytes();
            if !b.is_empty() && b[0] == b'-' {
            } else if rel.is_none() {
                rel = Some(String::from_utf8_lossy(b).into_owned());
            }
        }

        let kversion = rel.unwrap_or_else(get_kernel_release);
        let mod_dir = PathBuf::from(format!("/lib/modules/{}", kversion));
        let dep_file = mod_dir.join("modules.dep");

        if !mod_dir.exists() {
            eprintln!("depmod: directory {} does not exist", mod_dir.display());
            return Ok(1);
        }

        let mut ko_files = Vec::new();
        let mut stack = vec![mod_dir.clone()];
        while let Some(dir) = stack.pop() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if let Some(ext) = path.extension() {
                        if ext == "ko" || path.to_string_lossy().contains(".ko.") {
                            if let Ok(rel_path) = path.strip_prefix(&mod_dir) {
                                ko_files.push(rel_path.to_string_lossy().into_owned());
                            }
                        }
                    }
                }
            }
        }

        ko_files.sort();

        let mut out = Vec::new();
        for k in &ko_files {
            out.extend_from_slice(k.as_bytes());
            out.extend_from_slice(b":\n");
        }

        if let Err(e) = fs::write(&dep_file, &out) {
            eprintln!("depmod: cannot write {}: {}", dep_file.display(), e);
            return Ok(1);
        }

        Ok(0)
    }
}

pub struct ModinfoApplet;

impl Applet for ModinfoApplet {
    fn name(&self) -> &'static str {
        "modinfo"
    }
    fn description(&self) -> &'static str {
        "Display information about a Linux Kernel module"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut field_filter: Option<&[u8]> = None;
        let mut mod_target: Option<&Path> = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-F" && i + 1 < args.len() {
                i += 1;
                field_filter = Some(args[i].as_bytes());
            } else if !b.is_empty() && b[0] == b'-' {
            } else if mod_target.is_none() {
                mod_target = Some(Path::new(&args[i]));
            }
            i += 1;
        }

        let target = match mod_target {
            Some(t) => t,
            None => {
                eprintln!("modinfo: module name required");
                return Ok(1);
            }
        };

        let resolved_path = if target.exists() {
            target.to_path_buf()
        } else {
            let rel = get_kernel_release();
            let base = PathBuf::from(format!("/lib/modules/{}", rel));
            let name = target.to_string_lossy();
            let mut found = None;
            let mut stack = vec![base];
            while let Some(dir) = stack.pop() {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            stack.push(p);
                        } else if let Some(file_name) = p.file_name().and_then(|s| s.to_str()) {
                            if file_name == format!("{}.ko", name)
                                || file_name.starts_with(&format!("{}.ko.", name))
                            {
                                found = Some(p);
                                break;
                            }
                        }
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            match found {
                Some(p) => p,
                None => {
                    eprintln!("modinfo: module '{}' not found", target.display());
                    return Ok(1);
                }
            }
        };

        let data = match fs::read(&resolved_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("modinfo: {}: {}", resolved_path.display(), e);
                return Ok(1);
            }
        };

        let modinfo_strings = extract_elf_modinfo(&data);

        let mut out = Vec::new();
        if let Some(filt) = field_filter {
            for s in &modinfo_strings {
                if let Some((k, v)) = split_byte_slice(s, b'=') {
                    if k == filt {
                        out.extend_from_slice(v);
                        out.push(b'\n');
                    }
                }
            }
        } else {
            out.extend_from_slice(b"filename:       ");
            out.extend_from_slice(resolved_path.as_os_str().as_bytes());
            out.push(b'\n');
            for s in &modinfo_strings {
                if let Some((k, v)) = split_byte_slice(s, b'=') {
                    out.extend_from_slice(k);
                    out.push(b':');
                    let pad = if k.len() < 15 { 15 - k.len() } else { 1 };
                    out.resize(out.len() + pad, b' ');
                    out.extend_from_slice(v);
                    out.push(b'\n');
                }
            }
        }

        print_bytes(&out);
        Ok(0)
    }
}

fn extract_elf_modinfo(data: &[u8]) -> Vec<&[u8]> {
    let mut results = Vec::new();
    if data.len() < 64 || &data[0..4] != b"\x7fELF" {
        return results;
    }

    let is_64 = data[4] == 2;
    if is_64 {
        let e_shoff = u64::from_le_bytes(data[40..48].try_into().unwrap_or([0; 8])) as usize;
        let e_shentsize = u16::from_le_bytes(data[58..60].try_into().unwrap_or([0; 2])) as usize;
        let e_shnum = u16::from_le_bytes(data[60..62].try_into().unwrap_or([0; 2])) as usize;
        let e_shstrndx = u16::from_le_bytes(data[62..64].try_into().unwrap_or([0; 2])) as usize;

        if e_shstrndx < e_shnum && e_shoff + e_shnum * e_shentsize <= data.len() {
            let str_hdr_offset = e_shoff + e_shstrndx * e_shentsize;
            let str_offset = u64::from_le_bytes(
                data[str_hdr_offset + 24..str_hdr_offset + 32]
                    .try_into()
                    .unwrap_or([0; 8]),
            ) as usize;
            let str_size = u64::from_le_bytes(
                data[str_hdr_offset + 32..str_hdr_offset + 40]
                    .try_into()
                    .unwrap_or([0; 8]),
            ) as usize;

            if str_offset + str_size <= data.len() {
                let shstrtab = &data[str_offset..str_offset + str_size];

                for i in 0..e_shnum {
                    let h = e_shoff + i * e_shentsize;
                    let sh_name =
                        u32::from_le_bytes(data[h..h + 4].try_into().unwrap_or([0; 4])) as usize;
                    let sh_offset =
                        u64::from_le_bytes(data[h + 24..h + 32].try_into().unwrap_or([0; 8]))
                            as usize;
                    let sh_size =
                        u64::from_le_bytes(data[h + 32..h + 40].try_into().unwrap_or([0; 8]))
                            as usize;

                    if sh_name < shstrtab.len() {
                        let name_end = shstrtab[sh_name..]
                            .iter()
                            .position(|&b| b == 0)
                            .unwrap_or(shstrtab.len() - sh_name);
                        let name = &shstrtab[sh_name..sh_name + name_end];
                        if name == b".modinfo" && sh_offset + sh_size <= data.len() {
                            let sec_data = &data[sh_offset..sh_offset + sh_size];
                            for entry in sec_data.split(|&b| b == 0) {
                                if !entry.is_empty() {
                                    results.push(entry);
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
    results
}

pub struct AcpidApplet;

impl Applet for AcpidApplet {
    fn name(&self) -> &'static str {
        "acpid"
    }
    fn description(&self) -> &'static str {
        "Listen to ACPI events and spawn specific helper programs"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut foreground = false;
        let mut event_file: Option<&Path> = None;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-f" || b == b"--foreground" {
                foreground = true;
            } else if b == b"-e" || (!b.is_empty() && b[0] == b'-') {
            } else if event_file.is_none() {
                event_file = Some(Path::new(arg));
            }
        }

        let ev_path = event_file.unwrap_or_else(|| Path::new("/proc/acpi/event"));

        if !foreground {
            let pid = unsafe { libc::fork() };
            if pid < 0 {
                eprintln!("acpid: fork failed");
                return Ok(1);
            }
            if pid > 0 {
                return Ok(0);
            }
            unsafe {
                libc::setsid();
            }
        }

        let file = match File::open(ev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("acpid: cannot open {}: {}", ev_path.display(), e);
                return Ok(1);
            }
        };

        let reader = io::BufReader::new(file);
        for line in reader.lines().map_while(|l| l.ok()) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let handler = Path::new("/etc/acpi/handler.sh");
            if handler.exists() {
                let _ = std::process::Command::new(handler).args(&parts).status();
            }
        }

        Ok(0)
    }
}

pub struct LsofApplet;

impl Applet for LsofApplet {
    fn name(&self) -> &'static str {
        "lsof"
    }
    fn description(&self) -> &'static str {
        "List open files"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let mut out = Vec::new();
        out.extend_from_slice(b"COMMAND     PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME\n");

        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let pid_str = file_name.to_string_lossy();
                let pid: u32 = match pid_str.parse() {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                let proc_path = entry.path();
                let comm_bytes = fs::read(proc_path.join("comm")).unwrap_or_default();
                let comm = if let Some(i) = comm_bytes.iter().position(|&b| b == b'\n') {
                    &comm_bytes[..i]
                } else {
                    &comm_bytes[..]
                };

                let fd_dir = proc_path.join("fd");
                if let Ok(fds) = fs::read_dir(&fd_dir) {
                    for fd_entry in fds.flatten() {
                        let fd_name = fd_entry.file_name();
                        if let Ok(target) = fs::read_link(fd_entry.path()) {
                            out.extend_from_slice(comm);
                            if comm.len() < 10 {
                                out.resize(out.len() + 10 - comm.len(), b' ');
                            }
                            out.push(b' ');

                            put_num_pad_left(&mut out, pid as u64, 5);
                            out.extend_from_slice(b" root   ");

                            let fd_b = fd_name.as_bytes();
                            out.extend_from_slice(fd_b);
                            if fd_b.len() < 4 {
                                out.resize(out.len() + 4 - fd_b.len(), b' ');
                            }
                            out.extend_from_slice(b" REG        0,0        0    0 ");
                            out.extend_from_slice(target.as_os_str().as_bytes());
                            out.push(b'\n');
                        }
                    }
                }
            }
        }

        print_bytes(&out);
        Ok(0)
    }
}

pub struct LsscsiApplet;

impl Applet for LsscsiApplet {
    fn name(&self) -> &'static str {
        "lsscsi"
    }
    fn description(&self) -> &'static str {
        "List SCSI devices (or hosts) and their attributes"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let scsi_dir = Path::new("/sys/bus/scsi/devices");
        let mut out = Vec::new();

        if scsi_dir.exists() {
            if let Ok(entries) = fs::read_dir(scsi_dir) {
                let mut devs = Vec::new();
                for entry in entries.flatten() {
                    devs.push(entry.path());
                }
                devs.sort();

                for p in devs {
                    let dev_name = p.file_name().unwrap_or_default().to_string_lossy();

                    if !dev_name.contains(':') {
                        continue;
                    }

                    let vendor = fs::read_to_string(p.join("vendor")).unwrap_or_default();
                    let model = fs::read_to_string(p.join("model")).unwrap_or_default();
                    let dev_type = fs::read_to_string(p.join("type")).unwrap_or_default();
                    let type_str = match dev_type.trim() {
                        "0" => "disk   ",
                        "1" => "tape   ",
                        "4" => "worm   ",
                        "5" => "cd/dvd ",
                        _ => "process",
                    };

                    out.push(b'[');
                    out.extend_from_slice(dev_name.as_bytes());
                    out.extend_from_slice(b"]  ");
                    out.extend_from_slice(type_str.as_bytes());
                    out.push(b' ');
                    out.extend_from_slice(vendor.trim().as_bytes());
                    out.push(b' ');
                    out.extend_from_slice(model.trim().as_bytes());
                    out.push(b'\n');
                }
            }
        }

        if out.is_empty() {
            if let Ok(proc_scsi) = fs::read_to_string("/proc/scsi/scsi") {
                out.extend_from_slice(proc_scsi.as_bytes());
            }
        }

        print_bytes(&out);
        Ok(0)
    }
}

pub struct TreeApplet;

impl Applet for TreeApplet {
    fn name(&self) -> &'static str {
        "tree"
    }
    fn description(&self) -> &'static str {
        "List contents of directories in a tree-like format"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut all_files = false;
        let mut dirs_only = false;
        let mut root_dir: Option<&Path> = None;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-a" {
                all_files = true;
            } else if b == b"-d" {
                dirs_only = true;
            } else if !b.is_empty() && b[0] == b'-' {
            } else if root_dir.is_none() {
                root_dir = Some(Path::new(arg));
            }
        }

        let root = root_dir.unwrap_or_else(|| Path::new("."));
        let mut out = Vec::new();
        out.extend_from_slice(root.as_os_str().as_bytes());
        out.push(b'\n');

        let mut dir_count = 0u64;
        let mut file_count = 0u64;
        let mut prefix = Vec::new();

        traverse_tree(
            root,
            all_files,
            dirs_only,
            &mut prefix,
            &mut dir_count,
            &mut file_count,
            &mut out,
        );

        out.push(b'\n');
        put_num(&mut out, dir_count);
        out.extend_from_slice(if dir_count == 1 {
            b" directory"
        } else {
            b" directories"
        });
        if !dirs_only {
            out.extend_from_slice(b", ");
            put_num(&mut out, file_count);
            out.extend_from_slice(if file_count == 1 {
                b" file\n"
            } else {
                b" files\n"
            });
        } else {
            out.push(b'\n');
        }

        print_bytes(&out);
        Ok(0)
    }
}

fn traverse_tree(
    dir: &Path,
    all_files: bool,
    dirs_only: bool,
    prefix: &mut Vec<u8>,
    dir_count: &mut u64,
    file_count: &mut u64,
    out: &mut Vec<u8>,
) {
    let mut entries = Vec::new();
    if let Ok(rd) = fs::read_dir(dir) {
        for entry in rd.flatten() {
            let file_name = entry.file_name();
            let b = file_name.as_bytes();
            if !all_files && b.starts_with(b".") {
                continue;
            }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if dirs_only && !is_dir {
                continue;
            }
            entries.push((entry.path(), file_name, is_dir));
        }
    }

    entries.sort_by(|a, b| a.1.cmp(&b.1));

    let len = entries.len();
    for (i, (path, name, is_dir)) in entries.into_iter().enumerate() {
        let is_last = i + 1 == len;
        out.extend_from_slice(prefix);
        if is_last {
            out.extend_from_slice("└── ".as_bytes());
        } else {
            out.extend_from_slice("├── ".as_bytes());
        }
        out.extend_from_slice(name.as_bytes());
        out.push(b'\n');

        if is_dir {
            *dir_count += 1;
            let prev_len = prefix.len();
            if is_last {
                prefix.extend_from_slice(b"    ");
            } else {
                prefix.extend_from_slice("│   ".as_bytes());
            }
            traverse_tree(
                &path, all_files, dirs_only, prefix, dir_count, file_count, out,
            );
            prefix.truncate(prev_len);
        } else {
            *file_count += 1;
        }
    }
}

pub struct MimApplet;

impl Applet for MimApplet {
    fn name(&self) -> &'static str {
        "mim"
    }
    fn description(&self) -> &'static str {
        "Run executable script from memory"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("mim: file required");
            return Ok(1);
        }

        let script_file = Path::new(&args[0]);
        let data = match fs::read(script_file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("mim: can't read {}: {}", script_file.display(), e);
                return Ok(1);
            }
        };

        let name = CString::new("mim").unwrap();
        let mfd = unsafe { libc::syscall(libc::SYS_memfd_create, name.as_ptr(), 1) };
        if mfd < 0 {
            eprintln!("mim: memfd_create failed");
            return Ok(1);
        }

        let mut file = unsafe { File::from_raw_fd(mfd as i32) };
        if file.write_all(&data).is_err() {
            eprintln!("mim: write to memfd failed");
            return Ok(1);
        }

        let mut argv_c: Vec<CString> = Vec::new();
        argv_c.push(CString::new(args[0].as_bytes()).unwrap_or_default());
        for a in &args[1..] {
            argv_c.push(CString::new(a.as_bytes()).unwrap_or_default());
        }
        let mut argv_ptrs: Vec<*const libc::c_char> = argv_c.iter().map(|c| c.as_ptr()).collect();
        argv_ptrs.push(std::ptr::null());

        let env_c: Vec<CString> = std::env::vars()
            .map(|(k, v)| CString::new(format!("{}={}", k, v)).unwrap())
            .collect();
        let mut env_ptrs: Vec<*const libc::c_char> = env_c.iter().map(|c| c.as_ptr()).collect();
        env_ptrs.push(std::ptr::null());

        unsafe {
            libc::fexecve(file.as_raw_fd(), argv_ptrs.as_ptr(), env_ptrs.as_ptr());
        }

        let proc_fd_path = format!("/proc/self/fd/{}", file.as_raw_fd());
        let path_c = CString::new(proc_fd_path).unwrap();
        unsafe {
            libc::execve(path_c.as_ptr(), argv_ptrs.as_ptr(), env_ptrs.as_ptr());
        }

        let err = io::Error::last_os_error();
        eprintln!("mim: execution failed: {}", err);
        Ok(1)
    }
}

pub struct LoggerApplet;

impl Applet for LoggerApplet {
    fn name(&self) -> &'static str {
        "logger"
    }
    fn description(&self) -> &'static str {
        "Log a message to syslog"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut tag: Option<&[u8]> = None;
        let priority: libc::c_int = libc::LOG_USER | libc::LOG_NOTICE;
        let mut msg_parts: Vec<&[u8]> = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-t" && i + 1 < args.len() {
                i += 1;
                tag = Some(args[i].as_bytes());
            } else if b == b"-p" && i + 1 < args.len() {
                i += 1;
            } else if !b.is_empty() && b[0] == b'-' {
            } else {
                msg_parts.push(b);
            }
            i += 1;
        }

        let message = if msg_parts.is_empty() {
            let mut buf = Vec::new();
            let _ = io::stdin().read_to_end(&mut buf);
            buf
        } else {
            let mut buf = Vec::new();
            for (idx, part) in msg_parts.iter().enumerate() {
                if idx > 0 {
                    buf.push(b' ');
                }
                buf.extend_from_slice(part);
            }
            buf
        };

        let log_sock_path = Path::new("/dev/log");
        if log_sock_path.exists() {
            if let Ok(sock) = UnixDatagram::unbound() {
                let tag_str = tag.unwrap_or(b"logger");
                let mut formatted = Vec::new();
                formatted.extend_from_slice(b"<13>");
                formatted.extend_from_slice(tag_str);
                formatted.extend_from_slice(b": ");
                formatted.extend_from_slice(&message);
                if sock.send_to(&formatted, log_sock_path).is_ok() {
                    return Ok(0);
                }
            }
        }

        let tag_cstr = tag
            .and_then(|t| CString::new(t).ok())
            .unwrap_or_else(|| CString::new("logger").unwrap());
        let msg_cstr = CString::new(message).unwrap_or_else(|_| CString::new("").unwrap());

        unsafe {
            libc::openlog(tag_cstr.as_ptr(), libc::LOG_PID, libc::LOG_USER);
            libc::syslog(priority, c"%s".as_ptr(), msg_cstr.as_ptr());
            libc::closelog();
        }

        Ok(0)
    }
}
