use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct UbirenameApplet;
impl Applet for UbirenameApplet {
    fn name(&self) -> &'static str {
        "ubirename"
    }
    fn description(&self) -> &'static str {
        "Rename UBI volume (UBI_IOCRNVOL; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = String::from("/dev/ubi0");
        let mut pair: Option<(String, String)> = None;
        for a in args {
            let b = ab(a);
            if b.starts_with(b"-") {
                continue;
            }
            let s = String::from_utf8_lossy(b).into_owned();
            if s.contains('=') && pair.is_none() {
                let (o, n) = s.split_once('=').unwrap_or(("", ""));
                pair = Some((o.to_owned(), n.to_owned()));
            } else {
                dev = s;
            }
        }

        let positional: Vec<String> = args
            .iter()
            .filter(|a| !ab(a).starts_with(b"-"))
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        if pair.is_none() && positional.len() >= 3 {
            dev = positional[0].clone();
            pair = Some((positional[1].clone(), positional[2].clone()));
        }
        let (old, new) = match pair {
            Some(p) => p,
            None => {
                eprintln!("ubirename: usage: ubirename DEV old=new");
                return Ok(1);
            }
        };

        let ob = old.as_bytes();
        let nbv = new.as_bytes();
        let mut req = vec![0u8; 16 + 140 * 2];
        req[0..4].copy_from_slice(&2i32.to_le_bytes());
        req[16..18].copy_from_slice(&(ob.len().min(127) as i16).to_le_bytes());
        req[20..20 + ob.len().min(127)].copy_from_slice(&ob[..ob.len().min(127)]);
        req[156..158].copy_from_slice(&(nbv.len().min(127) as i16).to_le_bytes());
        req[160..160 + nbv.len().min(127)].copy_from_slice(&nbv[..nbv.len().min(127)]);
        Ok(ubi_vol_ioctl(&dev, 1, &req))
    }
}
