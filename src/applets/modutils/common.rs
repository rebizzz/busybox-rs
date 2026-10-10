use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsString};
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::os::unix::ffi::OsStrExt;

pub fn print_bytes(bytes: &[u8]) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(bytes);
}

pub fn put_num(buf: &mut Vec<u8>, mut n: u64) {
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

pub fn put_num_pad_left(buf: &mut Vec<u8>, n: u64, width: usize) {
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

pub fn split_byte_slice(s: &[u8], delim: u8) -> Option<(&[u8], &[u8])> {
    s.iter()
        .position(|&b| b == delim)
        .map(|pos| (&s[..pos], &s[pos + 1..]))
}

pub fn get_kernel_release() -> String {
    let mut uts: libc::utsname = unsafe { std::mem::zeroed() };
    if unsafe { libc::uname(&mut uts) } == 0 {
        let r = unsafe { CStr::from_ptr(uts.release.as_ptr()) };
        return r.to_string_lossy().into_owned();
    }
    "custom".to_string()
}

