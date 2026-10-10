use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;

applet!(
    IostatApplet,
    "iostat",
    "Report CPU and I/O statistics",
    run_iostat
);
fn run_iostat(args: &[OsString]) -> Result<i32> {
    let (mut cpu_only, mut dev_only) = (false, false);
    for a in args {
        let b = ab(a);
        if b == b"-c" {
            cpu_only = true;
        } else if b == b"-d" {
            dev_only = true;
        } else if b == b"-k" || b == b"-m" {
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            let mut ok = true;
            for &c in &b[1..] {
                if !matches!(c, b'V' | b'x' | b'z' | b'N' | b'y') {
                    ok = false;
                    break;
                }
            }
            if !ok {
                eprintln!("iostat: invalid option '{}'", lossy(a));
                return Ok(1);
            }
        }
    }
    let st = read_stat();
    let mut out = wlock();
    if !dev_only {
        let c = cpu_line(&st, b"cpu").unwrap_or_default();
        let tot: u64 = c.iter().sum();
        let idle = c.get(3).copied().unwrap_or(0) + c.get(4).copied().unwrap_or(0);
        let busy = tot.saturating_sub(idle);
        let pct = |v: u64| {
            if tot == 0 {
                0.0
            } else {
                v as f64 * 100.0 / tot as f64
            }
        };
        let _ = writeln!(out, "avg-cpu:  %user   %nice %system %iowait  %idle");
        let _ = writeln!(
            out,
            "          {:6.2} {:6.2} {:6.2} {:6.2} {:6.2}",
            pct(c.first().copied().unwrap_or(0)
                + busy.saturating_sub(
                    c.get(2).copied().unwrap_or(0) + c.first().copied().unwrap_or(0)
                )),
            pct(c.get(1).copied().unwrap_or(0)),
            pct(c.get(2).copied().unwrap_or(0)),
            pct(c.get(4).copied().unwrap_or(0)),
            pct(idle)
        );
    }
    if !cpu_only {
        let _ = writeln!(
            out,
            "Device:            tps    kB_read/s    kB_wrtn/s    kB_read    kB_wrtn"
        );
        if let Ok(dd) = std::fs::read("/proc/diskstats") {
            for line in dd.split(|&c| c == b'\n') {
                let f: Vec<&[u8]> = line
                    .split(|&c| c == b' ' || c == b'\t')
                    .filter(|x| !x.is_empty())
                    .collect();
                if f.len() < 14 {
                    continue;
                }
                let nm = String::from_utf8_lossy(f[2]).into_owned();
                let rd: u64 = String::from_utf8_lossy(f[5]).parse().unwrap_or(0);
                let wr: u64 = String::from_utf8_lossy(f[9]).parse().unwrap_or(0);
                let rs: u64 = String::from_utf8_lossy(f[3]).parse().unwrap_or(0);
                let ws: u64 = String::from_utf8_lossy(f[7]).parse().unwrap_or(0);
                let _ = writeln!(
                    out,
                    "{:<16} {:6.2} {:12} {:12} {:9} {:9}",
                    nm,
                    (rs + ws) as f64,
                    rd / 2,
                    wr / 2,
                    rd / 2,
                    wr / 2
                );
            }
        }
    }
    Ok(0)
}
