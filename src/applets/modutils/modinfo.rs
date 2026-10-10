use super::common::*;
use crate::core::{Applet, Result};
use std::collections::HashMap;
use std::ffi::{CStr, CString, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};

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
