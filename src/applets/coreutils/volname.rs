use super::common::*;
use crate::core::Result;
use std::ffi::OsString;

applet!(VolnameApplet, "volname", "Print volume label", run_volname);
fn run_volname(args: &[OsString]) -> Result<i32> {
    if args.len() != 1 {
        eprintln!("volname: needs exactly one device argument");
        return Ok(1);
    }
    match probe_vol(args[0].as_os_str()) {
        Some(vi) if !vi.label.is_empty() => {
            println!("{}", vi.label);
            Ok(0)
        }
        Some(_) => {
            eprintln!("volname: no label on '{}'", lossy(&args[0]));
            Ok(1)
        }
        None => {
            eprintln!("volname: can't read '{}'", lossy(&args[0]));
            Ok(1)
        }
    }
}
