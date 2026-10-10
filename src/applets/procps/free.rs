use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

pub struct FreeApplet;
impl Applet for FreeApplet {
    fn name(&self) -> &'static str {
        "free"
    }
    fn description(&self) -> &'static str {
        "Display free and used memory"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut div: u64 = 1;
        let mut human = false;
        for a in args {
            let b = a.as_bytes();
            if b == b"-b" || b == b"-k" {
                div = 1;
                human = false;
            } else if b == b"-m" {
                div = 1024;
                human = false;
            } else if b == b"-g" {
                div = 1024 * 1024;
                human = false;
            } else if b == b"-h" {
                human = true;
            }
        }
        let mut buf = [0u8; 4096];
        let n = read_small("/proc/meminfo", &mut buf);
        if n == 0 {
            eprintln!("free: cannot read /proc/meminfo");
            return Ok(1);
        }
        let mem = &buf[..n];
        let total = meminfo_val(mem, b"MemTotal");
        let free = meminfo_val(mem, b"MemFree");
        let shared = meminfo_val(mem, b"Shmem");
        let buffers = meminfo_val(mem, b"Buffers");
        let cached = meminfo_val(mem, b"Cached");
        let stotal = meminfo_val(mem, b"SwapTotal");
        let sfree = meminfo_val(mem, b"SwapFree");
        let used = total.saturating_sub(free);
        let sused = stotal.saturating_sub(sfree);

        fn scaled(v: u64, div: u64, human: bool, out: &mut Vec<u8>) {
            if human {
                let bytes = v.saturating_mul(1024);
                let (q, suf) = if bytes >= 1 << 30 {
                    (bytes / (1 << 30), "G")
                } else if bytes >= 1 << 20 {
                    (bytes / (1 << 20), "M")
                } else {
                    (bytes / (1 << 10), "K")
                };
                push_u64(out, q);
                out.extend_from_slice(suf.as_bytes());
            } else if div == 1 {
                push_u64(out, v);
            } else {
                push_u64(out, v.div_ceil(div));
            }
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut line: Vec<u8> = Vec::with_capacity(128);
        out.write_all(
            b"              total        used        free      shared     buffers      cached\n",
        )?;
        line.extend_from_slice(b"Mem:");
        for v in [total, used, free, shared, buffers, cached] {
            line.push(b' ');
            let mut cell: Vec<u8> = Vec::with_capacity(12);
            scaled(v, div, human, &mut cell);

            while cell.len() < 11 {
                line.push(b' ');
            }
            line.extend_from_slice(&cell);
        }
        line.push(b'\n');
        out.write_all(&line)?;
        line.clear();
        line.extend_from_slice(b"Swap:");
        let mut cell: Vec<u8> = Vec::with_capacity(12);
        for v in [stotal, sused, sfree] {
            cell.clear();
            scaled(v, div, human, &mut cell);
            while cell.len() < 11 {
                line.push(b' ');
            }
            line.extend_from_slice(&cell);
            line.push(b' ');
        }

        while line.last() == Some(&b' ') {
            line.pop();
        }
        line.push(b'\n');
        out.write_all(&line)?;
        out.flush()?;
        Ok(0)
    }
}
