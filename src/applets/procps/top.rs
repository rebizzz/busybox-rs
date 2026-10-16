use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

pub struct TopApplet;
#[allow(clippy::type_complexity)]
type TopRow = (u32, Vec<u8>, u8, f64, u64, u64, u64, Vec<u8>);
impl Applet for TopApplet {
    fn name(&self) -> &'static str {
        "top"
    }
    fn description(&self) -> &'static str {
        "Show running processes (batch snapshot)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut iters: u64 = 1;
        let mut delay_ms: u64 = 300;
        let mut delay_given = false;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-b" {
            } else if b == b"-n" {
                i += 1;
                if i >= args.len() {
                    eprintln!("top: -n needs a number");
                    return Ok(1);
                }
                iters = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(1);
            } else if b.starts_with(b"-n") && b.len() > 2 {
                iters = std::str::from_utf8(&b[2..])
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(1);
            } else if b == b"-d" {
                i += 1;
                if i >= args.len() {
                    eprintln!("top: -d needs a delay");
                    return Ok(1);
                }
                delay_ms = parse_delay(args[i].as_bytes());
                delay_given = true;
            } else if b.starts_with(b"-d") && b.len() > 2 {
                delay_ms = parse_delay(&b[2..]);
                delay_given = true;
            }
            i += 1;
        }
        if iters == 0 {
            iters = 1;
        }
        let sample_gap = if delay_given {
            delay_ms
        } else if iters == 1 {
            300
        } else {
            1000
        };
        let mut mem_total: u64 = 1;
        {
            let mut mb = [0u8; 4096];
            let n = read_small("/proc/meminfo", &mut mb);
            let b = &mb[..n];
            let mut k = 0;
            while k < b.len() {
                if b[k..].starts_with(b"MemTotal:") {
                    let mut j = k + 9;
                    while j < b.len() && !b[j].is_ascii_digit() {
                        j += 1;
                    }
                    mem_total = 0;
                    while j < b.len() && b[j].is_ascii_digit() {
                        mem_total = mem_total * 10 + (b[j] - b'0') as u64;
                        j += 1;
                    }
                    break;
                }
                while k < b.len() && b[k] != b'\n' {
                    k += 1;
                }
                k += 1;
            }
        }
        let page_kb = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64 / 1024;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for it in 0..iters {
            let t0 = read_cpu_total();
            let mut snap0: Vec<(u32, u64)> = Vec::new();
            for pid in proc_pids() {
                if let Some(s) = read_stat(pid) {
                    snap0.push((pid, s.utime + s.stime));
                }
            }
            if sample_gap > 0 {
                let ts = libc::timespec {
                    tv_sec: (sample_gap / 1000) as libc::time_t,
                    tv_nsec: ((sample_gap % 1000) * 1_000_000) as libc::c_long,
                };
                unsafe { libc::nanosleep(&ts, std::ptr::null_mut()) };
            }
            let t1 = read_cpu_total();
            let dt = t1.saturating_sub(t0).max(1);
            let mut rows: Vec<TopRow> = Vec::new();
            let mut cmd = [0u8; 512];
            for (pid, c0) in &snap0 {
                let st = match read_stat(*pid) {
                    Some(s) => s,
                    None => continue,
                };
                let dc = (st.utime + st.stime).saturating_sub(*c0);
                let pct = dc as f64 * 100.0 / dt as f64;
                let rss_kb = (st.rss_pages.max(0) as u64).saturating_mul(page_kb);
                let mempct = rss_kb as f64 * 100.0 / mem_total.max(1) as f64;
                let n = read_cmdline(*pid, &mut cmd);
                let disp = if n > 0 {
                    let end = cmd[..n].iter().position(|&c| c == 0).unwrap_or(n);
                    cmd[..end].to_vec()
                } else {
                    let mut v = Vec::with_capacity(st.comm.len() + 2);
                    v.push(b'[');
                    v.extend_from_slice(&st.comm);
                    v.push(b']');
                    v
                };
                let uid = read_uids(*pid).map(|t| t.0).unwrap_or(0);
                rows.push((
                    *pid,
                    user_name(uid),
                    st.state,
                    pct,
                    mempct as u64,
                    rss_kb,
                    st.vsize / 1024,
                    disp,
                ));
            }
            rows.sort_by(|a, b| {
                b.3.partial_cmp(&a.3)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.0.cmp(&b.0))
            });
            out.write_all(b"PID USER     S %CPU %MEM   VIRT   RES COMMAND\n")?;
            for (pid, user, state, pct, memp, res, virt, disp) in rows.iter() {
                let mut line = Vec::with_capacity(128);
                push_u64(&mut line, *pid as u64);
                line.push(b' ');
                let mut u = user.clone();
                u.truncate(8);
                line.extend_from_slice(&u);
                while line.len() < 14 {
                    line.push(b' ');
                }
                line.push(*state);
                line.push(b' ');
                let c10 = (*pct * 10.0) as u64;
                push_u64(&mut line, c10 / 10);
                line.push(b'.');
                line.push(b'0' + (c10 % 10) as u8);
                line.push(b' ');
                push_u64(&mut line, *memp);
                line.push(b' ');
                push_u64(&mut line, *virt);
                line.push(b' ');
                push_u64(&mut line, *res);
                line.push(b' ');
                let mut d = disp.clone();
                d.truncate(64);
                line.extend_from_slice(&d);
                line.push(b'\n');
                out.write_all(&line)?;
            }
            out.flush()?;
            if it + 1 < iters && delay_ms > 0 {
                let ts = libc::timespec {
                    tv_sec: (delay_ms / 1000) as libc::time_t,
                    tv_nsec: ((delay_ms % 1000) * 1_000_000) as libc::c_long,
                };
                unsafe { libc::nanosleep(&ts, std::ptr::null_mut()) };
            }
        }
        Ok(0)
    }
}
