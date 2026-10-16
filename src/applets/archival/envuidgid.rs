use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub struct EnvuidgidApplet;
impl Applet for EnvuidgidApplet {
    fn name(&self) -> &'static str {
        "envuidgid"
    }
    fn description(&self) -> &'static str {
        "Set $UID/$GID, drop privs to account, then exec prog"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("envuidgid: usage: envuidgid account prog...");
            return Ok(1);
        }
        let acct = args[0].to_string_lossy().into_owned();
        let (uid, gid) = match pw_lookup(&acct) {
            Some(x) => x,
            None => {
                eprintln!("envuidgid: unknown account '{acct}'");
                return Ok(111);
            }
        };
        let mut cmd = Command::new(&args[1]);
        cmd.args(&args[2..]);
        cmd.env("UID", uid.to_string());
        cmd.env("GID", gid.to_string());
        cmd.uid(uid).gid(gid);
        let err = cmd.exec();
        eprintln!("envuidgid: {}: {err}", args[1].to_string_lossy());
        Ok(1)
    }
}
