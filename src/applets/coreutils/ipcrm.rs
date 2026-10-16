use super::common::*;
use crate::core::Result;
use std::ffi::OsString;

applet!(IpcrmApplet, "ipcrm", "Remove IPC objects", run_ipcrm);
fn run_ipcrm(args: &[OsString]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("ipcrm: needs arguments (use -m/-s/-q ID or -M/-S/-Q KEY)");
        return Ok(1);
    }
    let mut rc = 0;
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        let (kind, by_key) = match b {
            b"-m" => (b'm', false),
            b"-s" => (b's', false),
            b"-q" => (b'q', false),
            b"-M" => (b'm', true),
            b"-S" => (b's', true),
            b"-Q" => (b'q', true),
            b"-a" => {
                for id in 0..64 {
                    unsafe {
                        libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut());
                        libc::semctl(id, 0, libc::IPC_RMID);
                        libc::msgctl(id, libc::IPC_RMID, std::ptr::null_mut());
                    }
                }
                i += 1;
                continue;
            }
            _ => {
                eprintln!("ipcrm: invalid option '{}'", lossy(&args[i]));
                return Ok(1);
            }
        };
        i += 1;
        if i >= args.len() {
            eprintln!("ipcrm: option needs an argument");
            return Ok(1);
        }
        let v = lossy(&args[i]);
        if let Err(e) = ipc_key_rm(kind, &v, by_key) {
            eprintln!("ipcrm: can't remove {} '{}': {}", lossy(&args[i - 1]), v, e);
            rc = 1;
        }
        i += 1;
    }
    Ok(rc)
}
