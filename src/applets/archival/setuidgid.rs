use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct SetuidgidApplet;
impl Applet for SetuidgidApplet {
    fn name(&self) -> &'static str {
        "setuidgid"
    }
    fn description(&self) -> &'static str {
        "Drop privs to account (setgid+setuid) then exec prog"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("setuidgid: usage: setuidgid account prog...");
            return Ok(1);
        }
        let acct = args[0].to_string_lossy().into_owned();
        let (user, group) = split_user_group(&acct);
        let (uid, mut gid) = match pw_lookup(&user) {
            Some(x) => x,
            None => {
                eprintln!("setuidgid: unknown account '{user}'");
                return Ok(111);
            }
        };
        if let Some(g) = group {
            match gr_lookup(&g) {
                Some(id) => gid = id,
                None => {
                    eprintln!("setuidgid: unknown group '{g}'");
                    return Ok(111);
                }
            }
        }
        Ok(exec_prog(&args[1], &args[2..], Some(uid), Some(gid)))
    }
}
