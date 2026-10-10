use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;

applet!(
    BlkidApplet,
    "blkid",
    "Locate/print block device attributes",
    run_blkid
);
fn run_blkid(args: &[OsString]) -> Result<i32> {
    let (mut tag, mut out_fmt, mut match_t) = (None::<String>, None::<String>, None::<String>);
    let mut devs: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-s" {
            i += 1;
            if i >= args.len() {
                eprintln!("blkid: -s needs an argument");
                return Ok(1);
            }
            tag = Some(lossy(&args[i]));
        } else if b.starts_with(b"-s") && b.len() > 2 {
            tag = Some(String::from_utf8_lossy(&b[2..]).into_owned());
        } else if b == b"-o" {
            i += 1;
            if i >= args.len() {
                eprintln!("blkid: -o needs an argument");
                return Ok(1);
            }
            out_fmt = Some(lossy(&args[i]));
        } else if b.starts_with(b"-o") && b.len() > 2 {
            out_fmt = Some(String::from_utf8_lossy(&b[2..]).into_owned());
        } else if b == b"-t" {
            i += 1;
            if i >= args.len() {
                eprintln!("blkid: -t needs an argument");
                return Ok(1);
            }
            match_t = Some(lossy(&args[i]));
        } else if b.starts_with(b"-t") && b.len() > 2 {
            match_t = Some(String::from_utf8_lossy(&b[2..]).into_owned());
        } else if b == b"-c" {
            i += 1;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("blkid: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else {
            devs.push(args[i].clone());
        }
        i += 1;
    }
    if devs.is_empty() {
        devs = candidate_devs();
    }
    let val_only = out_fmt.as_deref() == Some("value");
    let mut rc = 0;
    let mut found = false;
    let mut out = wlock();
    for d in &devs {
        let vi = match probe_vol(d.as_os_str()) {
            Some(v) => v,
            None => continue,
        };
        if let Some(m) = &match_t {
            let (k, vv) = match m.split_once('=') {
                Some(x) => x,
                None => continue,
            };
            let ok = match k {
                "LABEL" => vi.label == vv,
                "UUID" => vi.uuid.eq_ignore_ascii_case(vv),
                "TYPE" => vi.fstype == vv,
                _ => false,
            };
            if !ok {
                continue;
            }
        }
        found = true;
        if val_only {
            let s = match tag.as_deref() {
                Some("LABEL") => vi.label.clone(),
                Some("UUID") => vi.uuid.clone(),
                Some("TYPE") => vi.fstype.clone(),
                _ => vi.uuid.clone(),
            };
            let _ = writeln!(out, "{}", s);
        } else if let Some(t) = &tag {
            let s = match t.as_str() {
                "LABEL" => vi.label.clone(),
                "UUID" => vi.uuid.clone(),
                "TYPE" => vi.fstype.clone(),
                _ => String::new(),
            };
            let _ = writeln!(out, "{}: {}=\"{}\"", lossy(d), t, s);
        } else {
            let _ = writeln!(
                out,
                "{}: LABEL=\"{}\" UUID=\"{}\" TYPE=\"{}\"",
                lossy(d),
                vi.label,
                vi.uuid,
                vi.fstype
            );
        }
    }
    if match_t.is_some() && !found {
        rc = 2;
    }
    Ok(rc)
}
