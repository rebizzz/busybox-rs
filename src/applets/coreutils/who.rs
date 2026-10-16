use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct WhoApplet;

impl Applet for WhoApplet {
    fn name(&self) -> &'static str {
        "who"
    }
    fn description(&self) -> &'static str {
        "Show who is logged on"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        let tty = unsafe {
            let t = libc::ttyname(0);
            if !t.is_null() {
                let cs = CString::from_raw(t);
                let s = cs.to_string_lossy().into_owned();
                let _ = cs.into_raw();
                s.strip_prefix("/dev/").unwrap_or(&s).to_string()
            } else {
                "pts/0".to_string()
            }
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        unsafe {
            let mut tm: libc::tm = std::mem::zeroed();
            let t = now as libc::time_t;
            libc::localtime_r(&t, &mut tm);
            let mut buf = [0u8; 64];
            let cfmt = CString::new("%Y-%m-%d %H:%M").unwrap();
            let n = libc::strftime(
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.len(),
                cfmt.as_ptr(),
                &tm,
            );
            let date_str = std::str::from_utf8(&buf[..n]).unwrap_or("");
            println!("{:<8} {:<12} {}", user, tty, date_str);
        }

        Ok(0)
    }
}

