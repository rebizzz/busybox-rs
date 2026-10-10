use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self};
use std::mem;
use std::os::unix::ffi::OsStrExt;

pub struct VconfigApplet;
impl Applet for VconfigApplet {
    fn name(&self) -> &'static str {
        "vconfig"
    }
    fn description(&self) -> &'static str {
        "VLAN configuration utility"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: vconfig add <iface> <vlan_id> | rem <vlan_iface>");
            return Ok(1);
        }

        let cmd = args[0].to_string_lossy();
        let mut vargs: VlanIoctlArgs = unsafe { mem::zeroed() };

        match cmd.as_ref() {
            "add" => {
                if args.len() < 3 {
                    eprintln!("vconfig: add requires interface and vlan_id");
                    return Ok(1);
                }
                let iface = args[1].as_bytes();
                let vid: u32 = args[2].to_string_lossy().parse().unwrap_or(0);
                vargs.cmd = 0;
                let len = iface.len().min(23);
                for (k, &b) in iface[..len].iter().enumerate() {
                    vargs.device1[k] = b as libc::c_char;
                }
                vargs.u.vlan_id = vid;
            }
            "rem" => {
                let iface = args[1].as_bytes();
                vargs.cmd = 1;
                let len = iface.len().min(23);
                for (k, &b) in iface[..len].iter().enumerate() {
                    vargs.device1[k] = b as libc::c_char;
                }
            }
            _ => {
                eprintln!("vconfig: unknown command '{}'", cmd);
                return Ok(1);
            }
        }

        let fd = open_socket_dgram()?;
        let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
        ifr.ifr_ifru.ifru_data = &mut vargs as *mut _ as *mut libc::c_char;

        let ret = unsafe { libc::ioctl(fd, SIOCSIFVLAN as _, &ifr) };
        unsafe { libc::close(fd) };

        if ret < 0 {
            eprintln!("vconfig: ioctl error: {}", io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}
