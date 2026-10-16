use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct PivotRootApplet;
impl Applet for PivotRootApplet {
    fn name(&self) -> &'static str {
        "pivot_root"
    }
    fn description(&self) -> &'static str {
        "Change the root filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() != 2 {
            eprintln!("usage: pivot_root NEWROOT PUT_OLD");
            return Ok(1);
        }
        use std::ffi::CString;
        let new = CString::new(args[0].as_bytes()).unwrap_or_else(|_| CString::new("/").unwrap());
        let old = CString::new(args[1].as_bytes()).unwrap_or_else(|_| CString::new("/").unwrap());

        let r = unsafe {
            libc::syscall(
                libc::SYS_pivot_root as libc::c_long,
                new.as_ptr(),
                old.as_ptr(),
            )
        };
        if r != 0 {
            eprintln!("pivot_root: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}

fn chdir_cstr(p: &[u8]) -> bool {
    use std::ffi::CString;
    match CString::new(p) {
        Ok(c) => unsafe { libc::chdir(c.as_ptr()) == 0 },
        Err(_) => false,
    }
}
