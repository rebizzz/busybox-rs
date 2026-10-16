use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self};
use std::mem;
use std::os::unix::ffi::OsStrExt;

pub struct IfenslaveApplet;
impl Applet for IfenslaveApplet {
    fn name(&self) -> &'static str {
        "ifenslave"
    }
    fn description(&self) -> &'static str {
        "Attach network interfaces to a bonding device"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut detach = false;
        let mut master: Option<String> = None;
        let mut slaves = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-d" {
                detach = true;
            } else if master.is_none() {
                master = Some(s.to_string());
            } else {
                slaves.push(s.to_string());
            }
            i += 1;
        }

        let m = match master {
            Some(m) => m,
            None => {
                eprintln!("Usage: ifenslave [-d] <master> <slave>...");
                return Ok(1);
            }
        };

        let fd = open_socket_dgram()?;
        let req = if detach {
            SIOCBONDRELEASE
        } else {
            SIOCBONDENSLAVE
        };

        let mut status = 0;
        for slave in slaves {
            let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
            set_ifr_name(&mut ifr, m.as_bytes());

            let mut slave_ifr: libc::ifreq = unsafe { mem::zeroed() };
            set_ifr_name(&mut slave_ifr, slave.as_bytes());

            unsafe {
                let dst_ptr = &mut ifr.ifr_ifru.ifru_slave as *mut _ as *mut u8;
                let src_ptr = slave_ifr.ifr_name.as_ptr() as *const u8;
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, libc::IFNAMSIZ);
                if libc::ioctl(fd, req as _, &ifr) < 0 {
                    eprintln!(
                        "ifenslave: ioctl failed on master {} slave {}: {}",
                        m,
                        slave,
                        io::Error::last_os_error()
                    );
                    status = 1;
                }
            }
        }

        unsafe { libc::close(fd) };
        Ok(status)
    }
}
