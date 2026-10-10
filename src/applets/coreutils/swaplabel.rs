use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;

applet!(
    SwaplabelApplet,
    "swaplabel",
    "Print or change swap LABEL/UUID",
    run_swaplabel
);
fn run_swaplabel(args: &[OsString]) -> Result<i32> {
    let (mut want_l, mut want_u) = (false, false);
    let mut dev: Option<OsString> = None;
    for a in args {
        let b = ab(a);
        if b == b"-l" || b == b"--label" {
            want_l = true;
        } else if b == b"-U" || b == b"--uuid" {
            want_u = true;
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("swaplabel: invalid option '{}'", lossy(a));
            return Ok(1);
        } else if dev.is_none() {
            dev = Some(a.clone());
        } else {
            eprintln!("swaplabel: too many arguments");
            return Ok(1);
        }
    }
    let dev = match dev {
        Some(d) => d,
        None => {
            eprintln!("swaplabel: needs a device argument");
            return Ok(1);
        }
    };
    let vi = match probe_vol(dev.as_os_str()) {
        Some(v) if v.fstype == "swap" => v,
        _ => {
            eprintln!("swaplabel: '{}' is not a swap device", lossy(&dev));
            return Ok(1);
        }
    };
    let mut out = wlock();
    if want_l && !want_u {
        let _ = writeln!(out, "LABEL: {}", vi.label);
    } else if want_u && !want_l {
        let _ = writeln!(out, "UUID: {}", vi.uuid);
    } else {
        let _ = writeln!(out, "LABEL: {}", vi.label);
        let _ = writeln!(out, "UUID: {}", vi.uuid);
    }
    Ok(0)
}

