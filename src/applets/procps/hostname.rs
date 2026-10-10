use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct HostnameApplet;
impl Applet for HostnameApplet {
    fn name(&self) -> &'static str {
        "hostname"
    }
    fn description(&self) -> &'static str {
        "Get or set the hostname"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut opt_s = false;
        let mut opt_d = false;
        let mut opt_f = false;
        let mut opt_i = false;
        let mut set_file: Option<OsString> = None;
        let mut new_name: Option<OsString> = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                i += 1;
                if i < args.len() {
                    new_name = Some(args[i].clone());
                }
                break;
            }
            if b.starts_with(b"-") && b != b"-" {
                if b == b"-F" || b == b"--file" {
                    i += 1;
                    if i < args.len() {
                        set_file = Some(args[i].clone());
                    }
                } else if b.starts_with(b"-F") {
                    set_file = Some(OsStr::from_bytes(&b[2..]).to_os_string());
                } else {
                    for &c in &b[1..] {
                        match c {
                            b's' => opt_s = true,
                            b'd' => opt_d = true,
                            b'f' => opt_f = true,
                            b'i' => opt_i = true,
                            _ => {}
                        }
                    }
                }
            } else {
                new_name = Some(args[i].clone());
                break;
            }
            i += 1;
        }

        if let Some(file_path) = set_file {
            let content = std::fs::read_to_string(&file_path)?;
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    let first_word = trimmed.split_whitespace().next().unwrap_or("");
                    let b = first_word.as_bytes();
                    if unsafe { libc::sethostname(b.as_ptr() as *const libc::c_char, b.len()) } != 0 {
                        eprintln!("hostname: {}", std::io::Error::last_os_error());
                        return Ok(1);
                    }
                    break;
                }
            }
            return Ok(0);
        }

        if let Some(name) = new_name {
            let b = name.as_bytes();
            if unsafe { libc::sethostname(b.as_ptr() as *const libc::c_char, b.len()) } != 0 {
                eprintln!("hostname: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            return Ok(0);
        }

        let mut buf = [0u8; 256];
        if unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len() - 1) } != 0 {
            eprintln!("hostname: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len() - 1);
        let hostname_bytes = &buf[..len];

        let stdout = std::io::stdout();
        let mut out = stdout.lock();

        if opt_d || opt_f || opt_i || (opt_s && (opt_d || opt_f || opt_i)) {
            let c_host = CString::new(hostname_bytes).unwrap_or_default();
            let hp = unsafe { gethostbyname(c_host.as_ptr()) };
            if hp.is_null() {
                eprintln!("hostname: Host name lookup failure");
                return Ok(1);
            }
            unsafe {
                let canon_name = if !(*hp).h_name.is_null() {
                    CStr::from_ptr((*hp).h_name).to_bytes()
                } else {
                    hostname_bytes
                };
                let dot_pos = canon_name.iter().position(|&c| c == b'.');

                if opt_f {
                    out.write_all(canon_name)?;
                    out.write_all(b"\n")?;
                } else if opt_s {
                    let short = match dot_pos {
                        Some(pos) => &canon_name[..pos],
                        None => canon_name,
                    };
                    out.write_all(short)?;
                    out.write_all(b"\n")?;
                } else if opt_d {
                    if let Some(pos) = dot_pos {
                        out.write_all(&canon_name[pos + 1..])?;
                    }
                    out.write_all(b"\n")?;
                } else if opt_i {
                    if (*hp).h_length as usize == std::mem::size_of::<libc::in_addr>() {
                        let mut addr_list = (*hp).h_addr_list as *const *const libc::in_addr;
                        let mut first = true;
                        while !(*addr_list).is_null() {
                            let in_addr = &**addr_list;
                            let ip = Ipv4Addr::from(in_addr.s_addr.to_ne_bytes());
                            if !first {
                                out.write_all(b" ")?;
                            }
                            first = false;
                            out.write_all(ip.to_string().as_bytes())?;
                            addr_list = addr_list.add(1);
                        }
                    }
                    out.write_all(b"\n")?;
                }
            }
        } else if opt_s {
            let short = match hostname_bytes.iter().position(|&c| c == b'.') {
                Some(pos) => &hostname_bytes[..pos],
                None => hostname_bytes,
            };
            out.write_all(short)?;
            out.write_all(b"\n")?;
        } else {
            out.write_all(hostname_bytes)?;
            out.write_all(b"\n")?;
        }
        out.flush()?;
        Ok(0)
    }
}
