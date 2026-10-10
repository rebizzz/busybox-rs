use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;

applet!(
    MpstatApplet,
    "mpstat",
    "Report per-CPU statistics",
    run_mpstat
);
fn run_mpstat(args: &[OsString]) -> Result<i32> {
    let mut only: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-P" {
            i += 1;
            if i >= args.len() {
                eprintln!("mpstat: -P needs an argument");
                return Ok(1);
            }
            only = Some(lossy(&args[i]));
        } else if b.starts_with(b"-P") && b.len() > 2 {
            only = Some(String::from_utf8_lossy(&b[2..]).into_owned());
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("mpstat: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        }
        i += 1;
    }
    let st = read_stat();
    let mut out = wlock();
    let _ = writeln!(out, "CPU    %usr   %nice    %sys %iowait    %idle");
    for line in st.split(|&c| c == b'\n') {
        if !line.starts_with(b"cpu") {
            continue;
        }
        let is_all = line.starts_with(b"cpu ");
        let id = if is_all {
            "all".to_string()
        } else {
            String::from_utf8_lossy(
                &line[..line.iter().position(|&c| c == b' ').unwrap_or(line.len())],
            )
            .into_owned()
        };
        if let Some(f) = &only {
            if f != "ALL" && id != *f && !(is_all && f == "all") {
                continue;
            }
        }
        let nums: Vec<u64> = line
            .split(|&c| c == b' ' || c == b'\t')
            .skip(1)
            .filter(|x| !x.is_empty())
            .map(|x| String::from_utf8_lossy(x).parse().unwrap_or(0))
            .collect();
        let tot: u64 = nums.iter().sum();
        let pct = |v: u64| {
            if tot == 0 {
                0.0
            } else {
                v as f64 * 100.0 / tot as f64
            }
        };
        let g = |k: usize| nums.get(k).copied().unwrap_or(0);
        let _ = writeln!(
            out,
            "{:<6} {:6.2} {:6.2} {:6.2} {:6.2} {:6.2}",
            id,
            pct(g(0)),
            pct(g(1)),
            pct(g(2)),
            pct(g(4)),
            pct(g(3) + g(4))
        );
    }
    Ok(0)
}
