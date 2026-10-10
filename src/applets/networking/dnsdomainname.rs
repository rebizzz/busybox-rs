use crate::core::{Applet, Result};
use std::ffi::{CStr, OsString};

pub struct DnsdomainnameApplet;

impl Applet for DnsdomainnameApplet {
    fn name(&self) -> &'static str {
        "dnsdomainname"
    }
    fn description(&self) -> &'static str {
        "Show DNS domain name"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let mut buf = [0u8; 256];
        let res = unsafe { libc::getdomainname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
        if res == 0 {
            let name = unsafe { CStr::from_ptr(buf.as_ptr() as *const libc::c_char) };
            let s = name.to_string_lossy();
            if s != "(none)" {
                println!("{}", s);
                return Ok(0);
            }
        }

        let mut hbuf = [0u8; 256];
        let hres = unsafe { libc::gethostname(hbuf.as_mut_ptr() as *mut libc::c_char, hbuf.len()) };
        if hres == 0 {
            let hname = unsafe { CStr::from_ptr(hbuf.as_ptr() as *const libc::c_char) };
            let hs = hname.to_string_lossy();
            if let Some(idx) = hs.find('.') {
                println!("{}", &hs[idx + 1..]);
                return Ok(0);
            }
        }
        println!();
        Ok(0)
    }
}
